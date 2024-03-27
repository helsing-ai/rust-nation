pub mod control;
pub mod h264;
pub mod sensors;

pub const RCV_ADDR: [u8; 4] = [0, 0, 0, 0];
pub const RCV_PORT: u16 = 8890;
pub const SND_ADDR: [u8; 4] = [192, 168, 10, 1];
pub const SND_PORT: u16 = 8889;
pub const VID_ADDR: [u8; 4] = [0, 0, 0, 0];
pub const VID_PORT: u16 = 11111;
