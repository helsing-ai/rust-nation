use hackathon_vision::BoundingBox;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    /// Calculates the absolute distance to another point
    pub fn distance(&self, other: &Position) -> u32 {
        let x = (self.x as f32 - other.x as f32).abs().powi(2);
        let y = (self.y as f32 - other.y as f32).abs().powi(2);
        (x + y).sqrt() as u32
    }
}

pub(crate) fn distance(a: &BoundingBox, b: &BoundingBox) -> u32 {
    let a = Position::from(a.to_owned());
    let b = Position::from(b.to_owned());

    a.distance(&b)
}

impl From<BoundingBox> for Position {
    fn from(value: BoundingBox) -> Self {
        Self {
            x: (value.x_min() + value.x_max()).div_euclid(2),
            y: (value.y_min() + value.y_max()).div_euclid(2),
        }
    }
}
