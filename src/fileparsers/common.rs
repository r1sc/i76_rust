use core::f32;

use nom::{
    call, do_parse, map, named, named_args,
    number::complete::{le_f32, le_u32, le_u8},
    take, take_str,
};

use crate::math::Vec3;

named_args!(pub cstring(length: u32)<String>,
    map!(take!(length), |bytes| {
        let mut buffer = vec![];
        for c in bytes {
            if *c == 0 {
                break;
            }
            buffer.push(*c);
        }
        String::from_utf8(buffer).unwrap()
    })
);

#[derive(Debug, Clone, Copy)]
pub struct ColorRGB(u8, u8, u8);
impl super::common::Parsable<Self> for ColorRGB {
    named!(
        parse<ColorRGB>,
        do_parse!(r: le_u8 >> g: le_u8 >> b: le_u8 >> (ColorRGB(r, g, b)))
    );
}

#[derive(Clone, Copy)]
pub struct RotationAxis {
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
}
impl super::common::Parsable<Self> for RotationAxis {
    named!(
        parse<Self>,
        do_parse!(
            right: call!(Vec3::parse)
                >> (Self {
                    right,
                    up: right,
                    forward: right
                })
        )
    );
}


pub trait Parsable<T> {
    fn parse(input: &[u8]) -> Result<(&[u8], T), nom::Err<nom::error::Error<&[u8]>>>;
}
