use crate::raw::errors::MotorError;
use crate::raw::Motor;
use hal::I2cdev;
use linux_embedded_hal as hal;
use pwm_pca9685::{Channel, Pca9685};

// Constants for servo control
const SERVO_MIN_PULSE: u16 = 2458; // in microseconds
const SERVO_MAX_PULSE: u16 = 7377; // in microseconds
                                   // const SERVO_FREQUENCY: f32 = 50.0; // in Hertz
const ACTUATION_RANGE: f32 = 180.0;

pub struct ServoKit {
    channel: Channel,
}

impl ServoKit {
    pub fn try_new(pwm: &mut Pca9685<I2cdev>, motor: Motor) -> Result<Self, MotorError> {
        // Map the motor to the corresponding channel
        let channel = match motor {
            Motor::Servo => Channel::C0, // Example channel, adjust as necessary
            _ => return Err(MotorError::InvalidMotorError),
        };

        pwm.set_channel_on(channel, 0)
            .map_err(|_| MotorError::ChannelError)?;

        Ok(ServoKit { channel })
    }

    pub fn set_angle(&mut self, pwm: &mut Pca9685<I2cdev>, angle: f32) -> Result<(), MotorError> {
        if angle < 0.0 || angle > ACTUATION_RANGE {
            return Err(MotorError::InvalidAngle);
        };
        let fraction = angle / ACTUATION_RANGE;
        if !(fraction >= 0.0 && fraction <= 1.0) {
            return Err(MotorError::InvalidAngle);
        }
        let duty_cycle: u16 = (SERVO_MIN_PULSE as f32
            + (fraction * (SERVO_MAX_PULSE - SERVO_MIN_PULSE) as f32))
            as u16;
        println!(
            "Setting servo angle - angle: {angle}, fraction: {fraction}, duty_cycle: {duty_cycle}"
        );
        self.set_duty_cycle(pwm, duty_cycle)?;
        Ok(())
    }

    pub fn set_duty_cycle(
        &mut self,
        pwm: &mut Pca9685<I2cdev>,
        duty_cycle: u16,
    ) -> Result<(), MotorError> {
        let shifted_value = duty_cycle >> 4; // Shift by 4 bits for 12-bit resolution

        if duty_cycle == 0xFFFF {
            // Fully on
            pwm.set_channel_on(self.channel, 0x1000)
                .map_err(|_| MotorError::ChannelError)?;
            pwm.set_channel_off(self.channel, 0)
                .map_err(|_| MotorError::ChannelError)?;
        } else if duty_cycle < 0x0010 {
            // Fully off
            pwm.set_channel_on(self.channel, 0)
                .map_err(|_| MotorError::ChannelError)?;
            pwm.set_channel_off(self.channel, 0x1000)
                .map_err(|_| MotorError::ChannelError)?;
        } else {
            // Normal case
            pwm.set_channel_on(self.channel, 0)
                .map_err(|_| MotorError::ChannelError)?;
            pwm.set_channel_off(self.channel, shifted_value)
                .map_err(|_| MotorError::ChannelError)?;
        }

        Ok(())
    }
}
