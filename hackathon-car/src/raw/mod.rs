pub mod dc;
pub mod errors;
pub mod servo;

use crate::raw::dc::DcKit;
use crate::raw::servo::ServoKit;
use errors::MotorError;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Pca9685, SlaveAddr};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Hash, PartialEq, Eq)]
/// An enumeration of all potential motors that can be controlled via the
/// Motor HAT or the Servo HAT.
pub enum Motor {
    Motor1,
    Motor2,
    Motor3,
    Motor4,
    Stepper1,
    Stepper2,
    Servo,
}

/// Initializes the PWM to control the Motor HAT. This makes a few assumptions:
/// - Assumes only one Motor HAT as 0x96.
/// - Assumes only a pre-scale of 4 so the HAT is running at ~1600 Hz.
///
/// If no I2C bus is provided to the function, it will attempt to
/// connect to /dev/i2c-3 which will work for the nixified RPi.
/// Try /dev/i2c-1 for most other cases.
pub fn init_motor_pwm(i2c: Option<I2cdev>) -> Result<Pca9685<I2cdev>, MotorError> {
    let i2c = if let Some(i2c) = i2c {
        i2c
    } else {
        I2cdev::new("/dev/i2c-3").map_err(|_| MotorError::I2cError)?
    };

    // The default address for the motor hat is 96 (0x60).
    let address = SlaveAddr::Alternative(true, false, false, false, false, false);
    println!(
        "Connecting to DC motor at address: {:#x?}",
        address.address()
    );

    let mut pwm = Pca9685::new(i2c, address);
    pwm.enable().map_err(|_| MotorError::PwmError)?;
    pwm.set_prescale(4).map_err(|_| MotorError::PwmError)?;
    Ok(pwm)
}

/// Initializes the PWM to control the Servo HAT.
///
/// If no I2C bus is provided to the function, it will attempt to
/// connect to /dev/i2c-3 which will work for the nixified RPi.
/// Try /dev/i2c-1 for most other cases.
pub fn init_servo_pwm(i2c: Option<I2cdev>) -> Result<Pca9685<I2cdev>, MotorError> {
    let i2c = if let Some(i2c) = i2c {
        i2c
    } else {
        I2cdev::new("/dev/i2c-3").map_err(|_| MotorError::I2cError)?
    };

    let address = SlaveAddr::Alternative(false, false, false, false, false, false); // 0x40
    println!(
        "Connecting to servo motor at address: {:#x?}",
        address.address()
    );

    let mut pwm = Pca9685::new(i2c, address);
    println!("Enabling pwm");
    pwm.enable().map_err(|_| MotorError::PwmError)?;

    // Calculate prescale value for 50Hz
    let osc_clock = 25_000_000; // 25 MHz
    let pwm_freq = 50; // 50 Hz
    let prescale_value = (osc_clock as f32 / (4096.0 * pwm_freq as f32)).round() as u8 - 1;
    println!("Setting prescale to {}", prescale_value);
    pwm.set_prescale(prescale_value)
        .map_err(|_| MotorError::PwmError)?;

    Ok(pwm)
}

#[allow(dead_code)]
fn main() -> eyre::Result<()> {
    // DC Motor
    let mut dc_pwm = init_motor_pwm(None)?;
    let mut dc_motor = DcKit::try_new(&mut dc_pwm, Motor::Motor1)?;

    let _ = dc_motor.set_throttle(&mut dc_pwm, 1.0)?;
    sleep(Duration::from_secs(1));

    let _ = dc_motor.set_throttle(&mut dc_pwm, 0.0)?;
    sleep(Duration::from_secs(1));

    let _ = dc_motor.set_throttle(&mut dc_pwm, -1.0)?;
    sleep(Duration::from_secs(1));

    let _ = dc_motor.set_throttle(&mut dc_pwm, 0.0)?;
    dc_motor.stop(&mut dc_pwm)?;

    // Wait before turning
    sleep(Duration::from_secs(1));

    // Servo
    let mut servo_pwm = init_servo_pwm(None)?;
    let mut servo = ServoKit::try_new(&mut servo_pwm, Motor::Servo)?;
    // TODO: This doesn't work with a shorter 500ms waiting time!

    // Full Left
    let _ = servo.set_angle(&mut servo_pwm, 180.0);
    sleep(Duration::from_secs(1));

    // Full Right
    let _ = servo.set_angle(&mut servo_pwm, 0.0);
    sleep(Duration::from_secs(1));

    // Full Left
    let _ = servo.set_angle(&mut servo_pwm, 180.0);
    sleep(Duration::from_secs(1));

    // Neutral Position
    let _ = servo.set_angle(&mut servo_pwm, 90.0);
    Ok(())
}
