// The NAL units of an Annex B byte stream: the bytes between its start codes (00 00 01, or 00 00 00 01).
pub(crate) struct NalUnits<'a> {
    remaining: &'a [u8],
}

// H.265: the unit type sits in bits 1 to 6 of the first header byte.
const UNIT_TYPE_SHIFT: u8 = 1;
const UNIT_TYPE_MASK: u8 = 0x3F;

impl<'a> NalUnits<'a> {
    pub fn new(stream: &'a [u8]) -> NalUnits<'a> {
        let start = NalUnits::next_start(stream, 0).map_or(stream.len(), |(_, after)| after);

        NalUnits {
            remaining: &stream[start..],
        }
    }

    pub fn unit_type(unit: &[u8]) -> u8 {
        unit.first()
            .map_or(0, |header| (header >> UNIT_TYPE_SHIFT) & UNIT_TYPE_MASK)
    }

    // The position of the next start code at or after `from`, and the position just past it.
    fn next_start(stream: &[u8], from: usize) -> Option<(usize, usize)> {
        let mut index = from;

        while index + 3 <= stream.len() {
            if stream[index] == 0 && stream[index + 1] == 0 {
                if stream[index + 2] == 1 {
                    return Some((index, index + 3));
                }

                if stream[index + 2] == 0 && stream.get(index + 3) == Some(&1) {
                    return Some((index, index + 4));
                }
            }

            index += 1;
        }

        None
    }
}

impl<'a> Iterator for NalUnits<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.remaining.is_empty() {
            return None;
        }

        let (unit, rest) = match NalUnits::next_start(self.remaining, 0) {
            Some((end, after)) => (&self.remaining[..end], &self.remaining[after..]),
            None => (self.remaining, &self.remaining[self.remaining.len()..]),
        };
        self.remaining = rest;

        Some(unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_split_at_either_start_code() {
        let stream = [0, 0, 0, 1, 0x40, 1, 0, 0, 1, 0x42, 1, 2, 0, 0, 1, 0x44];
        let units: Vec<&[u8]> = NalUnits::new(&stream).collect();

        assert_eq!(units, vec![&[0x40, 1][..], &[0x42, 1, 2][..], &[0x44][..]]);
        assert_eq!(
            units.iter().map(|unit| NalUnits::unit_type(unit)).collect::<Vec<_>>(),
            vec![32, 33, 34]
        );
    }
}
