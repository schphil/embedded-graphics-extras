use embedded_graphics::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

pub fn rotate_point(point: Point, rotation: Rotation, center: Point) -> Point {
    let relative = point - center;
    let rotated = match rotation {
        Rotation::Rotate0 => relative,
        Rotation::Rotate90 => Point::new(-relative.y, relative.x),
        Rotation::Rotate180 => Point::new(-relative.x, -relative.y),
        Rotation::Rotate270 => Point::new(relative.y, -relative.x),
    };
    rotated + center
}
