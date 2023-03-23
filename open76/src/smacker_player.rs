//use std::{collections::VecDeque, fs::File, io::Read};

//use libsmacker::{EnableMask, Smk};
//use rodio::{buffer::SamplesBuffer, Sample, Sink, Source};

//use crate::{
//    gl::types::GLuint,
//    texture_loader::{gen_texture, update_texture},
//};

//pub struct SmackerPlayer {
//    smk: Smk,
//    pub texture: GLuint
//}

//#[derive(Debug)]
//pub enum SmackerError {
//    IoError(std::io::Error),
//    SmkError,
//}
//impl From<std::io::Error> for SmackerError {
//    fn from(e: std::io::Error) -> Self {
//        SmackerError::IoError(e)
//    }
//}

//#[derive(Debug)]
//pub enum TickError {
//    PaletteError,
//    VideoError,
//}

//impl SmackerPlayer {
//    pub fn new(path: &str) -> Result<Self, SmackerError> {
//        let mut f = File::open(path)?;
//        let mut buf: Vec<u8> = Vec::new();
//        f.read_to_end(&mut buf)?;
//        let smk = Smk::open_memory(&buf).ok_or(SmackerError::SmkError)?;
//        smk.enable(EnableMask::all());

//        smk.first().ok_or(SmackerError::SmkError)?;
//        let texture = gen_texture(smk.video_info.width, smk.video_info.height);

//        todo!();
//    }

//    pub fn tick(&mut self, delta_ms: f64) -> Result<(), TickError> {
//        let palette = self.smk.get_palette().ok_or(TickError::PaletteError)?;
//        let video = self.smk.get_video().ok_or(TickError::VideoError)?;

//        let size = (self.smk.video_info.width * self.smk.video_info.height) as usize;
//        let mut rgba_texture: Vec<u32> = Vec::with_capacity(size);
//        for i in 0..size {
//            let palette_index = video[i] as u32;
//            let r = palette[(palette_index * 3 + 0) as usize] as u32;
//            let g = palette[(palette_index * 3 + 1) as usize] as u32;
//            let b = palette[(palette_index * 3 + 2) as usize] as u32;
//            rgba_texture.push((255 << 24) | (r << 16) | (g << 8) | b);
//        }
//        update_texture(
//            self.smk.video_info.width,
//            self.smk.video_info.height,
//            &rgba_texture,
//        );

//        let audio = self.smk.get_audio(0).unwrap();
//        todo!();

//        self.smk.next().ok_or(TickError::VideoError)?;

//        Ok(())
//    }
//}
