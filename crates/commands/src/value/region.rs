use std::str::FromStr;

const POSITION_SEPARATOR: char = ',';
const SIZE_SEPARATOR: char = ':';
const DIMENSION_SEPARATOR: char = 'x';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl FromStr for Region {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("{text:?} is not a region like 100,50:800x600 (x,y:widthxheight in pixels)");
        let (position, size) = text.split_once(SIZE_SEPARATOR).ok_or_else(invalid)?;
        let (x, y) = position.split_once(POSITION_SEPARATOR).ok_or_else(invalid)?;
        let (width, height) = size.split_once(DIMENSION_SEPARATOR).ok_or_else(invalid)?;
        let region = Region {
            x: x.parse().map_err(|_| invalid())?,
            y: y.parse().map_err(|_| invalid())?,
            width: width.parse().map_err(|_| invalid())?,
            height: height.parse().map_err(|_| invalid())?,
        };

        if region.width == 0 || region.height == 0 {
            return Err(invalid());
        }

        Ok(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_parse_position_and_size() {
        let region: Region = "100,50:800x600".parse().unwrap();
        assert_eq!(
            region,
            Region {
                x: 100,
                y: 50,
                width: 800,
                height: 600
            }
        );
    }

    #[test]
    fn malformed_regions_are_rejected() {
        for text in ["", "100,50", "800x600", "a,b:cxd", "0,0:0x10", "-1,0:10x10"] {
            assert!(text.parse::<Region>().is_err(), "{text:?}");
        }
    }
}
