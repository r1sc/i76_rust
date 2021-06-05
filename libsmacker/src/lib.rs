use std::{os::raw::c_ulong, slice::from_raw_parts};

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_primitive;
use enum_primitive::FromPrimitive;
use libsmacker_sys as ffi;

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i8)]
pub enum SeekResult {
    Done = ffi::SMK_DONE,
    More = ffi::SMK_MORE,
    Last = ffi::SMK_LAST,
}
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum YScaleMode {
    None = ffi::SMK_FLAG_Y_NONE,
    Interlace = ffi::SMK_FLAG_Y_INTERLACE,
    Double = ffi::SMK_FLAG_Y_DOUBLE,
}
}

bitflags! {
    pub struct EnableMask: u8 {
        const TRACK0 = ffi::SMK_AUDIO_TRACK_0;
        const TRACK1 = ffi::SMK_AUDIO_TRACK_1;
        const TRACK2 = ffi::SMK_AUDIO_TRACK_2;
        const TRACK3 = ffi::SMK_AUDIO_TRACK_3;
        const TRACK4 = ffi::SMK_AUDIO_TRACK_4;
        const TRACK5 = ffi::SMK_AUDIO_TRACK_5;
        const TRACK6 = ffi::SMK_AUDIO_TRACK_6;
        const VIDEO_TRACK = ffi::SMK_VIDEO_TRACK;
    }
}

pub struct Smk {
    // This pointer must never be allowed to leave the struct
    ctx: *mut ffi::smk_t,
    pub stream_info: StreamInfo,
    pub video_info: VideoInfo,
    pub audio_info: AudioInfo,
}

pub struct StreamInfo {
    pub current_frame: u32,
    pub frame_count: u32,
    pub microseconds_per_frame: f64,
}

pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub y_scale_mode: YScaleMode,
}

pub struct AudioInfo {
    pub track_mask: EnableMask,
    pub channels: [u8; 7],
    pub bit_depth: [u8; 7],
    pub sample_rate: [u32; 7],
}

pub fn stream_info(ctx: *mut ffi::smk_t) -> Option<StreamInfo> {
    unsafe {
        let mut current_frame: u32 = 0;
        let mut frame_count: u32 = 0;
        let mut microseconds_per_frame: f64 = 0.0;
        let result = ffi::smk_info_all(
            ctx,
            &mut current_frame,
            &mut frame_count,
            &mut microseconds_per_frame,
        );
        if result == ffi::SMK_ERROR {
            None
        } else {
            Some(StreamInfo {
                current_frame,
                frame_count,
                microseconds_per_frame,
            })
        }
    }
}

pub fn video_info(ctx: *mut ffi::smk_t) -> Option<VideoInfo> {
    unsafe {
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut y_scale_mode: u8 = 0;
        let result = ffi::smk_info_video(ctx, &mut width, &mut height, &mut y_scale_mode);
        if result == ffi::SMK_ERROR {
            None
        } else {
            Some(VideoInfo {
                width,
                height,
                y_scale_mode: YScaleMode::from_u8(y_scale_mode).unwrap_or(YScaleMode::None),
            })
        }
    }
}

pub fn audio_info(ctx: *mut ffi::smk_t) -> Option<AudioInfo> {
    unsafe {
        let mut track_mask: u8 = 0;
        let mut channels: [u8; 7] = [0; 7];
        let mut bit_depth: [u8; 7] = [0; 7];
        let mut sample_rate: [u32; 7] = [0; 7];
        let result = ffi::smk_info_audio(
            ctx,
            &mut track_mask,
            channels.as_mut_ptr(),
            bit_depth.as_mut_ptr(),
            sample_rate.as_mut_ptr(),
        );
        if result == ffi::SMK_ERROR {
            None
        } else {
            Some(AudioInfo {
                track_mask: EnableMask::from_bits(track_mask).unwrap_or(EnableMask::empty()),
                channels,
                bit_depth,
                sample_rate,
            })
        }
    }
}

impl Smk {
    pub fn open_memory(buffer: &[u8]) -> Option<Self> {
        unsafe {
            let ctx = ffi::smk_open_memory(buffer.as_ptr(), buffer.len() as c_ulong);
            if ctx.is_null() {
                None
            } else {
                let stream_info = stream_info(ctx)?;
                let audio_info = audio_info(ctx)?;
                let video_info = video_info(ctx)?;

                Some(Self {
                    ctx,
                    stream_info,
                    audio_info,
                    video_info,
                })
            }
        }
    }

    pub fn get_palette(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = ffi::smk_get_palette(self.ctx);
            if ptr.is_null() {
                None
            } else {
                Some(from_raw_parts(ptr, 768))
            }
        }
    }

    pub fn get_video(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = ffi::smk_get_video(self.ctx);
            if ptr.is_null() {
                None
            } else {
                Some(from_raw_parts(
                    ptr,
                    (self.video_info.width * self.video_info.height) as usize,
                ))
            }
        }
    }

    pub fn get_audio(&self, track: u8) -> Option<&[u8]> {
        unsafe {
            let size = ffi::smk_get_audio_size(self.ctx, track);
            let ptr = ffi::smk_get_audio(self.ctx, track);

            if ptr.is_null() {
                None
            } else {
                Some(from_raw_parts(ptr, size as usize))
            }
        }
    }

    pub fn first(&self) -> Option<SeekResult> {
        unsafe { SeekResult::from_i8(ffi::smk_first(self.ctx)) }
    }

    pub fn next(&self) -> Option<SeekResult> {
        unsafe { SeekResult::from_i8(ffi::smk_next(self.ctx)) }
    }

    pub fn seek_keyframe(&self, frame: u32) -> Option<SeekResult> {
        unsafe { SeekResult::from_i8(ffi::smk_seek_keyframe(self.ctx, frame)) }
    }

    pub fn enable(&self, mask: EnableMask) {
        unsafe {
            ffi::smk_enable_all(self.ctx, mask.bits);
        }
    }
}
impl Drop for Smk {
    fn drop(&mut self) {
        unsafe { ffi::smk_close(self.ctx) }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
