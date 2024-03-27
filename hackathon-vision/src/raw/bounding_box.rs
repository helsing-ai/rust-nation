use eyre::ensure;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundingBox {
    x_min: u32,
    y_min: u32,
    x_max: u32,
    y_max: u32,
}

impl BoundingBox {
    pub fn new(x_min: u32, y_min: u32, x_max: u32, y_max: u32) -> eyre::Result<Self> {
        ensure!(x_min <= x_max, "x_min must be inferior or equal than x_max");
        ensure!(y_min <= y_max, "y_min must be inferior or equal than y_max");
        Ok(Self {
            x_min,
            y_min,
            x_max,
            y_max,
        })
    }

    pub fn x_min(&self) -> u32 {
        self.x_min
    }
    pub fn y_min(&self) -> u32 {
        self.y_min
    }
    pub fn x_max(&self) -> u32 {
        self.x_max
    }
    pub fn y_max(&self) -> u32 {
        self.y_max
    }

    pub fn set_coordinates(
        &mut self,
        x_min: u32,
        y_min: u32,
        x_max: u32,
        y_max: u32,
    ) -> eyre::Result<()> {
        ensure!(
            x_min <= self.x_max,
            "x_min must be inferior or equal than x_max"
        );
        self.x_min = x_min;
        ensure!(
            y_min <= self.y_max,
            "y_min must be inferior or equal than y_max"
        );
        self.y_min = y_min;
        ensure!(
            x_max >= self.x_min,
            "x_min must be inferior or equal than x_max"
        );
        self.x_max = x_max;
        ensure!(
            y_max >= self.y_min,
            "y_min must be inferior or equal than y_max"
        );
        self.y_max = y_max;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (self.x_min..self.x_max).flat_map(move |x| (self.y_min..self.y_max).map(move |y| (x, y)))
    }

    pub fn is_within_size_bounds(&self, min_size: (u32, u32), max_size: (u32, u32)) -> bool {
        !((self.x_max - self.x_min < min_size.0 || self.y_max - self.y_min < min_size.1)
            || (self.x_max - self.x_min > max_size.0 || self.y_max - self.y_min > max_size.1))
    }
}
