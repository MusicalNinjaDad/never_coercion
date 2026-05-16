#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(never_type)]

use std::str::FromStr;

enum HeaderErr {
    ParseError,
}

impl From<!> for HeaderErr {
    fn from(value: !) -> Self {
        match value {}
    }
}

enum DeviceType {
    AudioController,
    VideoController,
    Custom(String),
}

impl FromStr for DeviceType {
    // We have a `Custom` type so this will never fail
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let devicetype = match s {
            "AudioController" => Self::AudioController,
            "VideoController" => Self::VideoController,
            _ => Self::Custom(s.to_string()),
        };
        Ok(devicetype)
    }
}

trait Header {}

impl Header for DeviceType {}

trait HeaderExt
where
    Self: Sized,
{
    /// Parse data from a header line ()
    fn parse_header(header: &str) -> Result<Self, HeaderErr>;
}

impl<H, E> HeaderExt for H
where
    H: Header + FromStr<Err = E>,
    HeaderErr: From<E>,
{
    /// Parse data from a header line ()
    fn parse_header(header: &str) -> Result<H, HeaderErr> {
        let (data, checksum) = header.split_once(", sha:").ok_or(HeaderErr::ParseError)?;
        Ok(data.parse()?)
    }
}

fn main() {
    // let device =
    //     DeviceType::parse_header("AudioController, sha:040f4bf53d2ca137d6f767169cdb2fa62849b156");
}

// error[E0599]: the variant or associated item `parse_header` exists for enum `DeviceType`, but its trait bounds were not satisfied
//   --> examples/conversion.rs:57:21
//    |
// 11 | enum DeviceType {
//    | --------------- variant or associated item `parse_header` not found for this enum
// ...
// 57 |         DeviceType::parse_header("AudioController, sha:040f4bf53d2ca137d6f767169cdb2fa62849b156");
//    |                     ^^^^^^^^^^^^ variant or associated item cannot be called on `DeviceType` due to unsatisfied trait bounds
//    |
// note: the following trait bounds were not satisfied:
//       `&DeviceType: Header`
//       `&DeviceType: std::str::FromStr`
//       `&mut DeviceType: Header`
//       `&mut DeviceType: std::str::FromStr`
//       `<&DeviceType as std::str::FromStr>::Err = _`
//       `<&mut DeviceType as std::str::FromStr>::Err = _`
//   --> examples/conversion.rs:45:8
//    |
// 43 | impl<H, E> HeaderExt for H
//    |            ---------     -
// 44 | where
// 45 |     H: Header + FromStr<Err = E>,
//    |        ^^^^^^   ^^^^^^^^^^^^^^^^
//    |        |        |       |
//    |        |        |       unsatisfied trait bound introduced here
//    |        |        unsatisfied trait bound introduced here
//    |        unsatisfied trait bound introduced here
//    = help: items from traits can only be used if the trait is implemented and in scope
// note: `HeaderExt` defines an item `parse_header`, perhaps you need to implement it
//   --> examples/conversion.rs:35:1
//    |
// 35 | trait HeaderExt
//    | ^^^^^^^^^^^^^^^
