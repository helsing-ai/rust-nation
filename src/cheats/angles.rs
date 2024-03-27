use super::positioning::Position;

impl super::positioning::Position {
    /// Computes an angle between -180° and 180°
    ///
    /// ```text
    ///             0
    ///       -45   |    45
    ///         \       /
    ///    
    ///   -90 -           – 90
    ///    
    ///         /       \
    ///      -135   |   135
    ///            180
    /// ```
    ///
    /// In the above example `self` is in the center and the returned
    /// degree indicated the angle of the location of `other`
    pub fn angle(&self, other: &Self) -> f64 {
        if self == other {
            return 0.0;
        }

        let dx: f64 = other.x as f64 - self.x as f64;
        let dy: f64 = other.y as f64 - self.y as f64;

        let radians = dy.atan2(dx);
        let degrees = radians.to_degrees();

        let degrees = -degrees + 90.0;

        if degrees > 180.0 {
            return degrees - 360.0;
        }

        degrees
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector(pub i32, pub i32);

impl Vector {
    pub fn dotproduct(&self, other: Vector) -> i32 {
        self.0 * other.0 + self.1 * other.1
    }

    pub fn magnitude(&self) -> i32 {
        ((self.0 as f64).powi(2) + (self.1 as f64).powi(2)).sqrt() as i32
    }

    pub fn angle(&self, other: Vector) -> f64 {
        let cos_angle =
            self.dotproduct(other) as f64 / (self.magnitude() * other.magnitude()) as f64;

        let angle_radians = cos_angle.acos();

        // round to full degrees
        (angle_radians.to_degrees() as i64) as f64
    }
}

impl From<(Position, Position)> for Vector {
    fn from(value: (Position, Position)) -> Self {
        Self(
            value.1.x as i32 - value.0.x as i32,
            value.1.y as i32 - value.0.y as i32,
        )
    }
}

#[cfg(test)]
mod test {
    mod positions {
        use crate::cheats::positioning::Position;

        #[test]
        fn eq() {
            assert_eq!(Position { x: 0, y: 0 }.angle(&Position { x: 0, y: 0 }), 0.0);
        }

        #[test]
        fn top_right() {
            assert_eq!(
                Position { x: 0, y: 0 }.angle(&Position { x: 10, y: 10 }),
                45.0
            );
        }

        #[test]
        fn right() {
            assert_eq!(
                Position { x: 0, y: 0 }.angle(&Position { x: 10, y: 0 }),
                90.0
            );
        }

        #[test]
        fn bottom_right() {
            assert_eq!(
                Position { x: 10, y: 10 }.angle(&Position { x: 20, y: 0 }),
                135.0
            );
        }

        #[test]
        fn bottom() {
            assert_eq!(
                Position { x: 0, y: 10 }.angle(&Position { x: 0, y: 0 }),
                180.0
            );
        }

        #[test]
        fn bottom_left() {
            assert_eq!(
                Position { x: 10, y: 10 }.angle(&Position { x: 0, y: 0 }),
                -135.0
            );
        }

        #[test]
        fn left() {
            assert_eq!(
                Position { x: 10, y: 10 }.angle(&Position { x: 0, y: 10 }),
                -90.0
            );
        }

        #[test]
        fn top_left() {
            assert_eq!(
                Position { x: 10, y: 0 }.angle(&Position { x: 0, y: 10 }),
                -45.0
            );
        }
    }

    mod vectors {
        use crate::cheats::angles::Vector;

        #[test]
        fn dot_product() {
            let vector_a = Vector(2, 3);
            let vector_b = Vector(-1, 2);
            assert_eq!(vector_a.dotproduct(vector_b), 4);
        }

        #[test]
        fn magnitude() {
            let vector = Vector(3, 4);
            assert_eq!(vector.magnitude(), 5);
        }

        mod angles {
            use super::*;

            #[test]
            fn itself() {
                let vector = Vector(1, 1);
                assert_eq!(vector.angle(vector), 0.0);
            }

            #[test]
            fn opposite_directions() {
                let vector_a = Vector(1, 0);
                let vector_b = Vector(-1, 0);
                assert_eq!(vector_a.angle(vector_b), 180.0);
            }

            #[test]
            fn acute() {
                let vector_a = Vector(2, 2);
                let vector_b = Vector(2, 1);
                let angle = vector_a.angle(vector_b);
                assert!(
                    angle >= 0.0 && angle <= 90.0,
                    "Angle is not acute: {}",
                    angle
                );
            }

            #[test]
            fn obtuse() {
                let vector_a = Vector(1, 0);
                let vector_b = Vector(-2, 2);
                let angle = vector_a.angle(vector_b);
                assert!(
                    angle >= 90.0 && angle <= 180.0,
                    "Angle is not obtuse: {}",
                    angle
                );
            }

            #[test]
            fn right() {
                let vector_a = Vector(0, 3);
                let vector_b = Vector(3, 0);
                assert_eq!(vector_a.angle(vector_b), 90.0, "Angle is not 90 degrees");
            }

            #[test]
            fn straight_line() {
                let vector_a = Vector(5, 0);
                let vector_b = Vector(-5, 0);
                assert_eq!(vector_a.angle(vector_b), 180.0, "Angle is not 180 degrees");
            }

            #[test]
            fn zero() {
                let vector_a = Vector(10, 10);
                let vector_b = Vector(10, 10);
                assert_eq!(vector_a.angle(vector_b), 0.0, "Angle is not 0 degrees");
            }

            #[test]
            fn parallel() {
                let vector_a = Vector(1, 3);
                let vector_b = Vector(2, 6);
                assert_eq!(vector_a.angle(vector_b), 0.0, "Angle is not 0 degrees");
            }

            #[test]
            fn negative_coordinates() {
                let vector_a = Vector(-4, -3);
                let vector_b = Vector(-2, -6);
                let angle = vector_a.angle(vector_b);
                assert!(angle > 0.0, "Angle should be positive, but got {}", angle);
            }

            #[test]
            fn large_values() {
                let vector_a = Vector(10000, 0);
                let vector_b = Vector(0, 10000);
                assert_eq!(
                    vector_a.angle(vector_b),
                    90.0,
                    "Angle with large values is not 90 degrees"
                );
            }
        }
    }
}
