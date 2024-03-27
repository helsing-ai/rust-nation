pub use hs_hackathon_macros::main;

/// The prelude module to jumpstart your implementation
pub mod prelude {
    pub use crate::car::*;
    pub use crate::drone::*;
    pub use crate::vision::*;

    pub use eyre;
    pub use futures;
    pub use futures::StreamExt;
    pub use tokio;
    pub use tracing;
    pub use tracing_subscriber;
}

/// A library for controling the drone, as well as recieving frames from its camera
pub mod drone {
    pub use hs_hackathon_drone::*;
}

/// A hardware abstraction layer over the motor and wheels of the RC car
pub mod car {
    pub use hs_hackathon_car::{Angle, MotorSocket, Velocity, WheelOrientation};
}

/// A computer vision api to detect LEDs inside of video frames recieved from drones
pub mod vision {
    pub use hs_hackathon_vision::{detect, distance, BoundingBox, Color, Led, LedDetectionConfig};
}
