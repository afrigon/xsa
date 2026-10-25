use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

use ash::vk;

use crate::mp4_box::Mp4Box;
use crate::nal_units::NalUnits;
use crate::{EncodedFrame, VideoError};

// H.265 NAL unit types of the parameter sets the decoder configuration carries.
const VPS: u8 = 32;
const SPS: u8 = 33;
const PPS: u8 = 34;
const NAL_HEADER_SIZE: usize = 2;
// The general profile, tier and level in an SPS: after the parameter set id byte, profile space to level, 12 bytes.
const PROFILE_TIER_LEVEL: std::ops::Range<usize> = 1..13;
const LENGTH_SIZE: u32 = 4;
const MDAT_HEADER_SIZE: u64 = 16;
// A largesize box: a 32-bit size of 1 says a 64-bit size follows the type.
const LARGE_SIZE: u32 = 1;
// The identity transformation, in 16.16 and 2.30 fixed point.
const UNITY_MATRIX: [u32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000];
const FIXED_POINT_ONE: u32 = 0x0001_0000;
const FULL_VOLUME: u16 = 0x0100;
const SEVENTY_TWO_DPI: u32 = 0x0048_0000;
const UNDETERMINED_LANGUAGE: u16 = 0x55C4;
const COLOR_DEPTH: u16 = 0x0018;
const TRACK_ENABLED_IN_MOVIE: u32 = 0x3;
const LOCAL_DATA: u32 = 0x1;

/// Writes encoded frames into an MP4 file: the frames stream into the file as they come, and the index of their
/// sizes, positions and keyframes follows them when it finishes.
pub struct Mp4Writer {
    file: BufWriter<File>,
    extent: vk::Extent2D,
    fps: u32,
    parameter_sets: ParameterSets,
    samples: Vec<Sample>,
    position: u64,
}

struct ParameterSets {
    video: Vec<u8>,
    sequence: Vec<u8>,
    picture: Vec<u8>,
}

struct Sample {
    offset: u64,
    size: u32,
    keyframe: bool,
}

impl Mp4Writer {
    /// Creates `path` for frames of `extent` at `fps`, described by `headers`, a `VideoEncoder`'s VPS, SPS and PPS.
    pub fn create(path: &Path, extent: vk::Extent2D, fps: u32, headers: &[u8]) -> Result<Mp4Writer, VideoError> {
        let parameter_sets = ParameterSets::find(headers)?;
        let mut file = BufWriter::new(File::create(path)?);
        let mut file_type = Mp4Box::new(b"ftyp");
        file_type
            .bytes(b"isom")
            .u32(0x200)
            .bytes(b"isom")
            .bytes(b"iso2")
            .bytes(b"mp41");
        let file_type = file_type.finish();
        file.write_all(&file_type)?;
        file.write_all(&LARGE_SIZE.to_be_bytes())?;
        file.write_all(b"mdat")?;
        file.write_all(&0u64.to_be_bytes())?;

        Ok(Mp4Writer {
            file,
            extent,
            fps,
            parameter_sets,
            samples: Vec::new(),
            position: file_type.len() as u64 + MDAT_HEADER_SIZE,
        })
    }

    /// Appends a frame, its NAL units each prefixed by their length as MP4 stores them.
    pub fn write(&mut self, frame: &EncodedFrame) -> Result<(), VideoError> {
        let offset = self.position;
        let mut size = 0;

        for unit in NalUnits::new(&frame.bytes) {
            // The decoder configuration carries the parameter sets.
            if matches!(NalUnits::unit_type(unit), VPS | SPS | PPS) {
                continue;
            }

            self.file.write_all(&(unit.len() as u32).to_be_bytes())?;
            self.file.write_all(unit)?;
            size += LENGTH_SIZE + unit.len() as u32;
        }

        self.position += u64::from(size);
        self.samples.push(Sample {
            offset,
            size,
            keyframe: frame.keyframe,
        });

        Ok(())
    }

    /// Writes the index and closes the file.
    pub fn finish(mut self) -> Result<(), VideoError> {
        let data_start = self.position - self.samples.iter().map(|sample| u64::from(sample.size)).sum::<u64>();
        let header_start = data_start - MDAT_HEADER_SIZE;
        self.file.seek(SeekFrom::Start(header_start + 8))?;
        self.file.write_all(&(self.position - header_start).to_be_bytes())?;
        self.file.seek(SeekFrom::End(0))?;
        let movie = self.movie()?;
        self.file.write_all(&movie)?;
        self.file.flush()?;

        Ok(())
    }

    fn movie(&self) -> Result<Vec<u8>, VideoError> {
        let frames = self.samples.len() as u32;
        let mut header = Mp4Box::full(b"mvhd", 0, 0);
        header
            .u32(0)
            .u32(0)
            .u32(self.fps)
            .u32(frames)
            .u32(FIXED_POINT_ONE)
            .u16(FULL_VOLUME)
            .zeros(10);
        header.u32s(&UNITY_MATRIX);
        header.zeros(24).u32(2);
        let mut movie = Mp4Box::new(b"moov");
        movie.child(header).child(self.track()?);

        Ok(movie.finish())
    }

    fn track(&self) -> Result<Mp4Box, VideoError> {
        let frames = self.samples.len() as u32;
        let mut header = Mp4Box::full(b"tkhd", 0, TRACK_ENABLED_IN_MOVIE);
        header
            .u32(0)
            .u32(0)
            .u32(1)
            .u32(0)
            .u32(frames)
            .zeros(8)
            .u16(0)
            .u16(0)
            .u16(0)
            .u16(0);
        header.u32s(&UNITY_MATRIX);
        header.u32(self.extent.width << 16).u32(self.extent.height << 16);
        let mut media_header = Mp4Box::full(b"mdhd", 0, 0);
        media_header
            .u32(0)
            .u32(0)
            .u32(self.fps)
            .u32(frames)
            .u16(UNDETERMINED_LANGUAGE)
            .u16(0);
        let mut handler = Mp4Box::full(b"hdlr", 0, 0);
        handler.u32(0).bytes(b"vide").zeros(12).bytes(b"VideoHandler\0");
        let mut video_header = Mp4Box::full(b"vmhd", 0, 1);
        video_header.u16(0).zeros(6);
        let location = Mp4Box::full(b"url ", 0, LOCAL_DATA);
        let mut references = Mp4Box::full(b"dref", 0, 0);
        references.u32(1).child(location);
        let mut data_information = Mp4Box::new(b"dinf");
        data_information.child(references);
        let mut media_information = Mp4Box::new(b"minf");
        media_information
            .child(video_header)
            .child(data_information)
            .child(self.sample_table()?);
        let mut media = Mp4Box::new(b"mdia");
        media.child(media_header).child(handler).child(media_information);
        let mut track = Mp4Box::new(b"trak");
        track.child(header).child(media);

        Ok(track)
    }

    fn sample_table(&self) -> Result<Mp4Box, VideoError> {
        let frames = self.samples.len() as u32;
        let mut description = Mp4Box::full(b"stsd", 0, 0);
        description.u32(1).child(self.sample_entry()?);
        let mut times = Mp4Box::full(b"stts", 0, 0);
        times.u32(1).u32(frames).u32(1);
        let keyframes: Vec<u32> = self
            .samples
            .iter()
            .enumerate()
            .filter(|(_, sample)| sample.keyframe)
            .map(|(index, _)| index as u32 + 1)
            .collect();
        let mut sync = Mp4Box::full(b"stss", 0, 0);
        sync.u32(keyframes.len() as u32);

        for keyframe in keyframes {
            sync.u32(keyframe);
        }

        // Every sample is a chunk of its own.
        let mut chunks = Mp4Box::full(b"stsc", 0, 0);
        chunks.u32(1).u32(1).u32(1).u32(1);
        let mut sizes = Mp4Box::full(b"stsz", 0, 0);
        sizes.u32(0).u32(frames);
        let mut offsets = Mp4Box::full(b"co64", 0, 0);
        offsets.u32(frames);

        for sample in &self.samples {
            sizes.u32(sample.size);
            offsets.u64(sample.offset);
        }

        let mut table = Mp4Box::new(b"stbl");
        table
            .child(description)
            .child(times)
            .child(sync)
            .child(chunks)
            .child(sizes)
            .child(offsets);

        Ok(table)
    }

    fn sample_entry(&self) -> Result<Mp4Box, VideoError> {
        let mut entry = Mp4Box::new(b"hvc1");
        entry
            .zeros(6)
            .u16(1)
            .zeros(16)
            .u16(self.extent.width as u16)
            .u16(self.extent.height as u16)
            .u32(SEVENTY_TWO_DPI)
            .u32(SEVENTY_TWO_DPI)
            .u32(0)
            .u16(1)
            .zeros(32)
            .u16(COLOR_DEPTH)
            .u16(0xFFFF)
            .child(self.parameter_sets.decoder_configuration()?);

        Ok(entry)
    }
}

impl ParameterSets {
    fn find(headers: &[u8]) -> Result<ParameterSets, VideoError> {
        let unit = |unit_type| {
            NalUnits::new(headers)
                .find(|unit| NalUnits::unit_type(unit) == unit_type)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| VideoError::Unsupported(format!("stream headers without NAL unit type {unit_type}")))
        };

        Ok(ParameterSets {
            video: unit(VPS)?,
            sequence: unit(SPS)?,
            picture: unit(PPS)?,
        })
    }

    // The HEVCDecoderConfigurationRecord (ISO/IEC 14496-15): the SPS's profile, tier and level, the stream's layout
    // and the parameter sets themselves.
    fn decoder_configuration(&self) -> Result<Mp4Box, VideoError> {
        let payload = ParameterSets::without_emulation_prevention(&self.sequence[NAL_HEADER_SIZE..]);
        let profile_tier_level = payload
            .get(PROFILE_TIER_LEVEL)
            .ok_or_else(|| VideoError::Unsupported("an SPS shorter than its profile, tier and level".to_string()))?;
        let mut configuration = Mp4Box::new(b"hvcC");
        configuration
            .u8(1)
            .bytes(profile_tier_level)
            // No minimum spatial segmentation, unknown parallelism, 4:2:0, 8-bit luma and chroma.
            .u16(0xF000)
            .u8(0xFC)
            .u8(0xFC | 1)
            .u8(0xF8)
            .u8(0xF8)
            .u16(0)
            // One temporal layer, nested, and 4-byte NAL unit lengths.
            .u8((1 << 3) | (1 << 2) | (LENGTH_SIZE as u8 - 1))
            .u8(3);

        for unit in [&self.video, &self.sequence, &self.picture] {
            // Every parameter set of this type is in the record.
            configuration
                .u8(0x80 | NalUnits::unit_type(unit))
                .u16(1)
                .u16(unit.len() as u16)
                .bytes(unit);
        }

        Ok(configuration)
    }

    // A NAL unit never contains 00 00 0x (x ≤ 3); the encoder inserts 03 after 00 00 to avoid it.
    fn without_emulation_prevention(bytes: &[u8]) -> Vec<u8> {
        let mut payload = Vec::with_capacity(bytes.len());
        let mut zeros = 0;

        for &byte in bytes {
            if zeros >= 2 && byte == 3 {
                zeros = 0;
                continue;
            }

            zeros = if byte == 0 { zeros + 1 } else { 0 };
            payload.push(byte);
        }

        payload
    }
}
