// One ISO base media box being written: a 32-bit size, a four-character type, then its contents.
pub(crate) struct Mp4Box {
    bytes: Vec<u8>,
}

impl Mp4Box {
    pub fn new(kind: &[u8; 4]) -> Mp4Box {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(kind);

        Mp4Box { bytes }
    }

    // A full box: a plain box whose contents start with a version byte and 24 bits of flags.
    pub fn full(kind: &[u8; 4], version: u8, flags: u32) -> Mp4Box {
        let mut full = Mp4Box::new(kind);
        full.u32((u32::from(version) << 24) | (flags & 0x00FF_FFFF));

        full
    }

    pub fn u8(&mut self, value: u8) -> &mut Mp4Box {
        self.bytes.push(value);
        self
    }

    pub fn u16(&mut self, value: u16) -> &mut Mp4Box {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u32(&mut self, value: u32) -> &mut Mp4Box {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u64(&mut self, value: u64) -> &mut Mp4Box {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u32s(&mut self, values: &[u32]) -> &mut Mp4Box {
        for value in values {
            self.u32(*value);
        }

        self
    }

    pub fn bytes(&mut self, value: &[u8]) -> &mut Mp4Box {
        self.bytes.extend_from_slice(value);
        self
    }

    pub fn zeros(&mut self, count: usize) -> &mut Mp4Box {
        self.bytes.resize(self.bytes.len() + count, 0);
        self
    }

    pub fn child(&mut self, child: Mp4Box) -> &mut Mp4Box {
        self.bytes.extend_from_slice(&child.finish());
        self
    }

    pub fn finish(mut self) -> Vec<u8> {
        let size = self.bytes.len() as u32;
        self.bytes[..4].copy_from_slice(&size.to_be_bytes());

        self.bytes
    }
}
