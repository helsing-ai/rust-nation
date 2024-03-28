#[cfg(target_os = "linux")]
mod raw;

#[cfg(target_os = "linux")]
use linux_embedded_hal::I2cdev;
#[cfg(target_os = "linux")]
use pwm_pca9685::Pca9685;
#[cfg(target_os = "linux")]
use raw::dc::DcKit;
#[cfg(target_os = "linux")]
use raw::servo::ServoKit;
#[cfg(target_os = "linux")]
use raw::{init_motor_pwm, init_servo_pwm, Motor};
#[cfg(target_os = "linux")]
use std::ops::Drop;

use std::time::{Duration, Instant};

#[derive(PartialEq, PartialOrd)]
pub struct Velocity(f32);

impl Velocity {
    pub fn backward() -> Self {
        Self(-100.0)
    }

    pub fn none() -> Self {
        Self(0.0)
    }

    pub fn forward() -> Self {
        Self(100.0)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for Velocity {
    type Error = eyre::Report;

    fn try_from(value: f32) -> eyre::Result<Velocity> {
        eyre::ensure!(
            value >= Velocity::backward().0 && value <= Velocity::forward().0,
            "velocity must be between {} and {}",
            Velocity::backward().0,
            Velocity::forward().0,
        );
        Ok(Velocity(value))
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self(0.0)
    }
}

/// The motor of the car
pub struct MotorSocket {
    #[cfg(target_os = "linux")]
    dc_motor: DcKit,
    #[cfg(target_os = "linux")]
    dc_pwm: Pca9685<I2cdev>,
    cooldown_since: Instant,
}

impl MotorSocket {
    /// Open the pins to talk to the car's motor
    pub async fn open() -> eyre::Result<Self> {
        #[cfg(target_os = "linux")]
        let mut dc_pwm = init_motor_pwm(None)?;
        #[cfg(target_os = "linux")]
        let dc_motor = DcKit::try_new(&mut dc_pwm, Motor::Motor1)?;
        Ok(MotorSocket {
            #[cfg(target_os = "linux")]
            dc_pwm,
            #[cfg(target_os = "linux")]
            dc_motor,
            cooldown_since: Instant::now(),
        })
    }

    /// Set the velocity of the motor
    ///
    /// Car will not move for longer than 1 second.
    pub async fn move_for(&mut self, velocity: Velocity, max_dur: Duration) -> eyre::Result<()> {
        if let Some(left) = Duration::from_secs(4).checked_sub(self.cooldown_since.elapsed()) {
            tokio::time::sleep(left).await;
        }
        // -1 because we wired it backwards
        #[cfg(target_os = "linux")]
        self.dc_motor
            .set_throttle(&mut self.dc_pwm, -1. * velocity.into_inner() / 100.0)?;
        #[cfg(not(target_os = "linux"))]
        let _ = velocity; // just for unused variables
        let actual_dur = std::cmp::min(Duration::from_secs(1), max_dur);
        tokio::time::sleep(actual_dur).await;
        #[cfg(target_os = "linux")]
        self.dc_motor
            .set_throttle(&mut self.dc_pwm, Velocity::none().into_inner() / 100.0)?;
        self.cooldown_since = Instant::now();
        Ok(())
    }

    /// Stop the motor.
    ///
    /// Note that once this is called, the motor will not start again without re-initialization.
    pub fn stop(&mut self) -> eyre::Result<()> {
        #[cfg(target_os = "linux")]
        self.dc_motor.stop(&mut self.dc_pwm)?;
        Ok(())
    }
}

impl Drop for MotorSocket {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Discrete wrapper of the wheel orientation
pub struct WheelOrientation {
    #[cfg(target_os = "linux")]
    servo: ServoKit,
    #[cfg(target_os = "linux")]
    servo_pwm: Pca9685<I2cdev>,
    current: Angle,
}

impl WheelOrientation {
    /// Open the pins to talk to the car's servo
    pub async fn new() -> eyre::Result<Self> {
        #[cfg(target_os = "linux")]
        let mut servo_pwm = init_servo_pwm(None)?;
        #[cfg(target_os = "linux")]
        let servo = ServoKit::try_new(&mut servo_pwm, Motor::Servo)?;
        Ok(WheelOrientation {
            #[cfg(target_os = "linux")]
            servo_pwm,
            #[cfg(target_os = "linux")]
            servo,
            current: Default::default(),
        })
    }

    /// Retrieve the current orientation of the wheels
    pub async fn current(self) -> eyre::Result<Angle> {
        Ok(self.current)
    }

    /// Set the wheel orientation to a specific value
    pub async fn set(&mut self, angle: Angle) -> eyre::Result<()> {
        self.current = angle;
        let raw: f32 = (-90.0 * angle.into_inner() + 90.0).try_into()?;
        #[cfg(not(target_os = "linux"))]
        let _ = raw;
        #[cfg(target_os = "linux")]
        self.servo.set_angle(&mut self.servo_pwm, raw)?;
        Ok(())
    }
}

impl Drop for WheelOrientation {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        let _ = self.servo.set_angle(&mut self.servo_pwm, 90.0);
    }
}

#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct Angle(f32);

impl Angle {
    pub fn left() -> Self {
        Self(-1.0)
    }

    pub fn straight() -> Self {
        Self(0.0)
    }

    pub fn right() -> Self {
        Self(1.0)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for Angle {
    type Error = eyre::Report;

    fn try_from(value: f32) -> eyre::Result<Angle> {
        eyre::ensure!(
            value >= Angle::left().0 && value <= Angle::right().0,
            "angle must be between {} and {}",
            Angle::left().0,
            Angle::right().0,
        );
        Ok(Angle(value))
    }
}

impl Default for Angle {
    fn default() -> Self {
        Self::straight()
    }
}
