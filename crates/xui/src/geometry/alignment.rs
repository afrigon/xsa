use super::{HorizontalAlignment, Point, Size, VerticalAlignment};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

impl Alignment {
    pub const TOP_LEADING: Alignment = Alignment::new(HorizontalAlignment::Leading, VerticalAlignment::Top);
    pub const TOP: Alignment = Alignment::new(HorizontalAlignment::Center, VerticalAlignment::Top);
    pub const TOP_TRAILING: Alignment = Alignment::new(HorizontalAlignment::Trailing, VerticalAlignment::Top);
    pub const LEADING: Alignment = Alignment::new(HorizontalAlignment::Leading, VerticalAlignment::Center);
    pub const CENTER: Alignment = Alignment::new(HorizontalAlignment::Center, VerticalAlignment::Center);
    pub const TRAILING: Alignment = Alignment::new(HorizontalAlignment::Trailing, VerticalAlignment::Center);
    pub const BOTTOM_LEADING: Alignment = Alignment::new(HorizontalAlignment::Leading, VerticalAlignment::Bottom);
    pub const BOTTOM: Alignment = Alignment::new(HorizontalAlignment::Center, VerticalAlignment::Bottom);
    pub const BOTTOM_TRAILING: Alignment = Alignment::new(HorizontalAlignment::Trailing, VerticalAlignment::Bottom);

    pub const fn new(horizontal: HorizontalAlignment, vertical: VerticalAlignment) -> Alignment {
        Alignment { horizontal, vertical }
    }

    pub fn position(self, container: Size, content: Size) -> Point {
        Point {
            x: self.horizontal.offset(container.width, content.width),
            y: self.vertical.offset(container.height, content.height),
        }
    }
}
