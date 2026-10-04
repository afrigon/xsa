use super::{EdgeInsets, Point, Size};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn inset(self, insets: EdgeInsets) -> Rect {
        Rect {
            origin: Point {
                x: self.origin.x + insets.leading,
                y: self.origin.y + insets.top,
            },
            size: Size {
                width: (self.size.width - insets.horizontal()).max(0.0),
                height: (self.size.height - insets.vertical()).max(0.0),
            },
        }
    }
}
