use hs_hackathon_vision::Color;

/// Math utilities for doing angle calculations and determining the rotation need to get towards
/// the target
pub mod angles;

/// Utilities for positioning, distance calculations and conversion
pub mod positioning;

/// Turning algorithm for turning a car towards a specific angle
pub mod turning;

/// Idling your car onto the target, until the target is changed
pub mod idling;

/// Approach the target with your car
pub mod approaching;

/// Holds your team's car and target color
pub struct TeamColors {
    /// The color of the car
    pub car: Color,
    /// The color of the target
    pub target: Color,
}

/// Contains internal helpers across cheats
#[doc(hidden)]
pub(crate) mod internal;
