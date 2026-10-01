use std::f32::consts::{PI, TAU};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::thread;

use anyhow::{Context, bail};
use glam::Vec3;
use image::Rgb32FImage;

const FACE_SIZE: usize = 4096;
const SAMPLES_PER_AXIS: usize = 2;
const DEFAULT_EXPOSURE: f32 = 1.0;
const USAGE: &str = "usage: convert_skybox <input.exr> <output.dds> [exposure]";
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;

type Face = Vec<Vec3>;

struct Arguments {
    input: String,
    output: String,
    exposure: f32,
}

fn parse_arguments() -> anyhow::Result<Arguments> {
    let mut arguments = std::env::args().skip(1);
    let Some(input) = arguments.next() else {
        bail!(USAGE);
    };
    let Some(output) = arguments.next() else {
        bail!(USAGE);
    };
    let exposure = match arguments.next() {
        Some(exposure) => exposure.parse().context("parsing the exposure")?,
        None => DEFAULT_EXPOSURE,
    };
    Ok(Arguments {
        input,
        output,
        exposure,
    })
}

fn main() -> anyhow::Result<()> {
    let Arguments {
        input,
        output,
        exposure,
    } = parse_arguments()?;

    let mut reader = image::ImageReader::open(&input).with_context(|| format!("opening {input}"))?;
    reader.no_limits();
    let map = reader
        .decode()
        .with_context(|| format!("reading {input}"))?
        .into_rgb32f();
    print_brightness_percentiles(&map);

    let faces: Vec<Face> = thread::scope(|scope| {
        let workers: Vec<_> = (0..6)
            .map(|face| {
                let map = &map;
                scope.spawn(move || project_face(map, face))
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("face worker panicked"))
            .collect()
    });
    let mip_chains: Vec<Vec<Face>> = faces.into_iter().map(mip_chain).collect();

    let average = mip_chains.iter().map(|chain| chain[chain.len() - 1][0]).sum::<Vec3>() / 6.0;
    println!("average linear color before exposure: {average}");
    println!(
        "average linear color after exposure: {}",
        (average * exposure).min(Vec3::ONE)
    );

    write_cube_dds(&output, &mip_chains, exposure).with_context(|| format!("writing {output}"))
}

fn print_brightness_percentiles(map: &Rgb32FImage) {
    let mut luminance: Vec<f32> = map
        .pixels()
        .step_by(97)
        .map(|pixel| 0.2126 * pixel[0] + 0.7152 * pixel[1] + 0.0722 * pixel[2])
        .collect();
    luminance.sort_by(f32::total_cmp);
    for percentile in [50.0, 90.0, 99.0, 99.9, 99.99] {
        let index = ((percentile / 100.0) * (luminance.len() - 1) as f32) as usize;
        println!("luminance p{percentile}: {}", luminance[index]);
    }
}

fn project_face(map: &Rgb32FImage, face: usize) -> Face {
    let mut texels = Vec::with_capacity(FACE_SIZE * FACE_SIZE);
    for row in 0..FACE_SIZE {
        for column in 0..FACE_SIZE {
            let mut sum = Vec3::ZERO;
            for sample_row in 0..SAMPLES_PER_AXIS {
                for sample_column in 0..SAMPLES_PER_AXIS {
                    let s = (column as f32 + (sample_column as f32 + 0.5) / SAMPLES_PER_AXIS as f32) / FACE_SIZE as f32;
                    let t = (row as f32 + (sample_row as f32 + 0.5) / SAMPLES_PER_AXIS as f32) / FACE_SIZE as f32;
                    sum += sample_equirectangular(map, face_direction(face, s * 2.0 - 1.0, t * 2.0 - 1.0));
                }
            }
            texels.push(sum / (SAMPLES_PER_AXIS * SAMPLES_PER_AXIS) as f32);
        }
    }
    texels
}

// Inverse of the cube map face selection table in the Vulkan specification.
fn face_direction(face: usize, s: f32, t: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -t, -s),
        1 => Vec3::new(-1.0, -t, s),
        2 => Vec3::new(s, 1.0, t),
        3 => Vec3::new(s, -1.0, -t),
        4 => Vec3::new(s, -t, 1.0),
        _ => Vec3::new(-s, -t, -1.0),
    }
    .normalize()
}

fn sample_equirectangular(map: &Rgb32FImage, direction: Vec3) -> Vec3 {
    let right_ascension = direction.y.atan2(direction.x);
    let declination = direction.z.clamp(-1.0, 1.0).asin();
    let u = (0.5 - right_ascension / TAU).rem_euclid(1.0);
    let v = 0.5 - declination / PI;

    let width = map.width() as usize;
    let height = map.height() as usize;
    let x = u * width as f32 - 0.5;
    let y = (v * height as f32 - 0.5).clamp(0.0, (height - 1) as f32);
    let x0 = x.floor();
    let y0 = y.floor();
    let fraction_x = x - x0;
    let fraction_y = y - y0;
    let texel = |x: f32, y: f32| {
        let column = (x as isize).rem_euclid(width as isize) as u32;
        let row = (y as usize).min(height - 1) as u32;
        let pixel = map.get_pixel(column, row);
        Vec3::new(pixel[0], pixel[1], pixel[2])
    };
    let top = texel(x0, y0).lerp(texel(x0 + 1.0, y0), fraction_x);
    let bottom = texel(x0, y0 + 1.0).lerp(texel(x0 + 1.0, y0 + 1.0), fraction_x);
    top.lerp(bottom, fraction_y)
}

fn mip_chain(face: Face) -> Vec<Face> {
    let mut chain = vec![face];
    let mut size = FACE_SIZE;
    while size > 1 {
        let previous = &chain[chain.len() - 1];
        let half = size / 2;
        let mut level = Vec::with_capacity(half * half);
        for row in 0..half {
            for column in 0..half {
                let at = |r: usize, c: usize| previous[(row * 2 + r) * size + column * 2 + c];
                level.push((at(0, 0) + at(0, 1) + at(1, 0) + at(1, 1)) / 4.0);
            }
        }
        chain.push(level);
        size = half;
    }
    chain
}

fn write_cube_dds(path: &str, mip_chains: &[Vec<Face>], exposure: f32) -> anyhow::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    let mip_count = mip_chains[0].len() as u32;
    let size = FACE_SIZE as u32;

    let mut header = Vec::with_capacity(148);
    header.extend_from_slice(b"DDS ");
    for value in [
        124,
        0x1 | 0x2 | 0x4 | 0x1000 | 0x20000 | 0x8,
        size,
        size,
        size * 4,
        0,
        mip_count,
    ] {
        header.extend_from_slice(&u32::to_le_bytes(value));
    }
    header.extend_from_slice(&[0; 44]);
    for value in [32, 0x4] {
        header.extend_from_slice(&u32::to_le_bytes(value));
    }
    header.extend_from_slice(b"DX10");
    header.extend_from_slice(&[0; 20]);
    for value in [0x1000 | 0x8 | 0x400000, 0x200 | 0xFC00, 0, 0, 0] {
        header.extend_from_slice(&u32::to_le_bytes(value));
    }
    for value in [DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, 3, 0x4, 1, 0] {
        header.extend_from_slice(&u32::to_le_bytes(value));
    }
    file.write_all(&header)?;

    for chain in mip_chains {
        for level in chain {
            let bytes: Vec<u8> = level
                .iter()
                .flat_map(|color| {
                    let [red, green, blue] = (*color * exposure).to_array().map(encode_srgb);
                    [red, green, blue, 255]
                })
                .collect();
            file.write_all(&bytes)?;
        }
    }
    file.flush()?;
    Ok(())
}

fn encode_srgb(linear: f32) -> u8 {
    let linear = linear.clamp(0.0, 1.0);
    let encoded = if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}
