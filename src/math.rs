use nom::{do_parse, named, number::complete::le_f32};

#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub f32, pub f32, pub f32);
impl super::common::Parsable<Self> for Vec3 {
    named!(
        parse<Self>,
        do_parse!(x: le_f32 >> y: le_f32 >> z: le_f32 >> (Vec3(x, y, z)))
    );
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4(pub f32, pub f32, pub f32, pub f32);
impl Vec4 {
    named!(
        pub parse<Vec4>,
        do_parse!(x: le_f32 >> y: le_f32 >> z: le_f32 >> w: le_f32 >> (Vec4(x, y, z, w)))
    );
}
