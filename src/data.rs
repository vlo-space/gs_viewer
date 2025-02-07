use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReadConfidence {
    Unreliable = 0,
    Low = 1,
    Medium = 2,
    High = 3
}

impl TryFrom<u8> for ReadConfidence {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ReadConfidence::Unreliable,
            1 => ReadConfidence::Low,
            2 => ReadConfidence::Medium,
            3 => ReadConfidence::High,
            _ => {return Err(())}
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SensedData {
    pub index: u32,
    pub uptime: u32,

    pub temperature: f32,
    pub pressure: f32,

    pub vibration: u16,

    pub acceleration: [f64; 3],
    pub acceleration_confidence: ReadConfidence,

    pub gps_time: u32,
    pub gps_date: u32,
    pub gps_position: [f64; 2],
    pub gps_altitude: f64
}

#[derive(Debug, PartialEq)]
pub enum LogReadError {
    ParseError { msg: String, value: Option<String> },
}

pub fn parse_log_line(text: &str) -> Result<SensedData, LogReadError> {
    let text = text.trim_start().trim_end();

    let mut iterator = text.split('\t');

    // fn try_parse_next<T>(iter: &mut dyn Iterator<Item = &str>) -> Result<T, ()> where T: FromStr {
    //     iter.next().unwrap().parse::<T>()
    // }

    fn try_parse_next<T: std::str::FromStr>(iter: & mut dyn Iterator<Item = & str>) -> Result<T, LogReadError> where <T as FromStr>::Err: Display {
        let value = iter.next().unwrap();
        match value.parse::<T>() {
            Ok(v) => Ok(v),
            Err(e) => {
                Err(LogReadError::ParseError { msg: e.to_string(), value: Some(value.to_string()) })
            }
        }
    }

    fn try_parse_confidence_value(iter: & mut dyn Iterator<Item = & str>) -> Result<ReadConfidence, LogReadError> {
        ReadConfidence::try_from(try_parse_next::<u8>(iter)?)
            .map_err(|_| LogReadError::ParseError { msg: "Invalid confidence value".to_owned(), value: None })
    }

    let index: u32          = try_parse_next(&mut iterator)?;
    let uptime: u32         = try_parse_next(&mut iterator)?;
    let _micros: u32         = try_parse_next(&mut iterator)?;
    let temperature: f32    = try_parse_next(&mut iterator)?;
    let pressure: f32       = try_parse_next(&mut iterator)?;
    let vibration: u16      = try_parse_next(&mut iterator)?;
    let acceleration: [f64; 3] = [
        try_parse_next(&mut iterator)?,
        try_parse_next(&mut iterator)?,
        try_parse_next(&mut iterator)?
    ];
    let acceleration_confidence = try_parse_confidence_value(&mut iterator)?;
    let _gyroscope: [f64; 3] = [
        try_parse_next(&mut iterator)?,
        try_parse_next(&mut iterator)?,
        try_parse_next(&mut iterator)?
    ];
    let _gyroscope_confidence = try_parse_confidence_value(&mut iterator)?;
    let gps_time: u32       = try_parse_next(&mut iterator)?;
    let gps_date: u32       = try_parse_next(&mut iterator)?;
    let gps_position: [f64; 2] = [
        try_parse_next(&mut iterator)?,
        try_parse_next(&mut iterator)?
    ];
    let gps_altitude: f64   = try_parse_next(&mut iterator)?;

    return Ok(SensedData {
        index,
        uptime,
        temperature,
        pressure,
        vibration,
        acceleration,
        acceleration_confidence,
        gps_time,
        gps_date,
        gps_position,
        gps_altitude,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_log_line() {
        assert_eq!(
            parse_log_line("0\t1\t1\t2\t3\t4\t5\t5\t5\t0\t7\t7\t7\t0\t8\t9\t10\t10\t11"),
            Ok(SensedData {
                index: 0,
                uptime: 1,
                temperature: 2.0,
                pressure: 3.0,
                vibration: 4,
                acceleration: [5.0, 5.0, 5.0],
                acceleration_confidence: ReadConfidence::Unreliable,
                gps_time: 8,
                gps_date: 9,
                gps_position: [10.0, 10.0],
                gps_altitude: 11.0
            })
        );
    }

    #[test]
    fn test_read_log_line_nan() {
        let value = parse_log_line("0\t1\t1\t2\t3\t4\tnan\tnan\tnan\t0\t7\t7\t7\t0\t8\t9\t10\t10\t11").unwrap();
        assert_eq!(value.acceleration[0].is_nan(), true);
        assert_eq!(value.acceleration[1].is_nan(), true);
        assert_eq!(value.acceleration[2].is_nan(), true);
    }

    #[test]
    fn test_read_log_line_real_data() {
        assert_eq!(
            parse_log_line("32\t3770\t559\t24.59\t100349.23\t38\t-0.039063\t-0.652344\t9.730469\t2\tNaN\tnan\tnan\t0\t301124\t19284800\t0.000000\t0.000000\t0.000000")
                .is_ok(),
            true
        );
    }
}