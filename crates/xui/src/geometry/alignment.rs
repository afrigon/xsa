use super::{HorizontalAlignment, Point, Size, VerticalAlignment};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

impl Alignment {
    pub const TOP: Alignment = Alignment {
        horizontal: HorizontalAlignment::Center,
        vertical: VerticalAlignment::Top,
    };
    pub const CENTER: Alignment = Alignment {
        horizontal: HorizontalAlignment::Center,
        vertical: VerticalAlignment::Center,
    };

    pub fn position(self, container: Size, content: Size) -> Point {
        Point {
            x: self.horizontal.offset(container.width, content.width),
            y: self.vertical.offset(container.height, content.height),
        }
    }
}
