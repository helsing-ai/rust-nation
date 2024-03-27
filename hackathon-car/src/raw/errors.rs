use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MotorError {
    /// An error occurred initializing the I2C bus.
    I2cError,
    /// An error occurred configuring the PCA9685.
    PwmError,
    /// An error occurred setting a channel.
    ChannelError,
    /// The value for throttle is not in the bounds of [-1.0, 1.0].
    ThrottleError,
    /// An invalid motor was provided to a constructor, i.e. a stepper motor
    /// passed into the DcMotor constructor.
    InvalidMotorError,
    PositionError,
    InvalidAngle,
}

impl fmt::Display for MotorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MotorError {}

#[derive(Debug)]
pub enum ServoError {
    /// An error occurred initializing the I2C bus.
    I2cError,
    /// An error occurred configuring the PCA9685.
    PwmError,
}

impl fmt::Display for ServoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ServoError {}
