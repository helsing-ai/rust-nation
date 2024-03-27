use sscanf::sscanf;
use std::str::FromStr;
use tracing::{trace, warn};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub pitch: i32,
    // the degree of the attitude pitch.
    pub roll: i32,
    // the degree of the attitude roll.
    pub yaw: i32,
    // the degree of the attitude yaw.
    pub vgx: i32,
    // the speed of “x” axis.
    pub vgy: i32,
    // the speed of the “y” axis.
    pub vgz: i32,
    // the speed of the “z” axis.
    pub templ: i32,
    // the lowest temperature in degree Celsius.
    pub temph: i32,
    // the highest temperature in degree Celsius
    pub tof: i32,
    // the time of flight distance in cm.
    pub h: i32,
    // the height in cm.
    pub bat: i32,
    // the percentage of the current battery level.
    pub baro: f32,
    // the barometer measurement in cm.
    pub time: i32,
    // the amount of time the motor has been used.
    pub agx: f32,
    // the acceleration of the “x” axis.
    pub agy: f32,
    // the acceleration of the “y” axis.
    pub agz: f32, // the acceleration of the “z” axis.
}

impl FromStr for State {
    type Err = ();
    fn from_str(received: &str) -> Result<Self, Self::Err> {
        trace!("received: {}", &received);
        if let Ok((
                  pitch,
                  roll,
                  yaw,
                  vgx,
                  vgy,
                  vgz,
                  templ,
                  temph,
                  tof,
                  h,
                  bat,
                  baro,
                  time,
                  agx,
                  agy,
                  agz,
              )) = sscanf!(received.trim(), "pitch:{i32};roll:{i32};yaw:{i32};vgx:{i32};vgy:{i32};vgz:{i32};templ:{i32};temph:{i32};tof:{i32};h:{i32};bat:{i32};baro:{f32};time:{i32};agx:{f32};agy:{f32};agz:{f32};") {
        Ok(State {
            pitch,
            roll,
            yaw,
            vgx,
            vgy,
            vgz,
            templ,
            temph,
            tof,
            h,
            bat,
            baro,
            time,
            agx,
            agy,
            agz,
        })
    } else {
        warn!("unclear how to parse {received:?}");
        Err(())
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_example() {
        let msg = "pitch:0;roll:0;yaw:-93;vgx:0;vgy:0;vgz:0;templ:71;temph:74;tof:10;h:0;bat:32;baro:-33.48;time:0;agx:8.00;agy:0.00;agz:-1002.00;\r\n";
        let parsed = msg.parse::<State>();
        assert_eq!(71, parsed.unwrap().templ);
    }

    #[test]
    fn test_parse_without_trailing_newlines() {
        let msg = "pitch:0;roll:0;yaw:-93;vgx:0;vgy:0;vgz:0;templ:71;temph:74;tof:10;h:0;bat:32;baro:-33.48;time:0;agx:8.00;agy:0.00;agz:-1002.00;";
        let parsed = msg.parse::<State>();
        assert_eq!(71, parsed.unwrap().templ);
    }

    #[test]
    fn test_parse_zeros() {
        let msg = "pitch:0;roll:0;yaw:0;vgx:0;vgy:0;vgz:0;templ:0;temph:0;tof:0;h:0;bat:0;baro:0.0;time:0;agx:0.0;agy:0.0;agz:0.0;\r\n";
        let parsed = msg.parse::<State>();
        assert_eq!(0, parsed.unwrap().templ);
    }
}
