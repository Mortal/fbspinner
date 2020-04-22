//! Simple Linux framebuffer abstraction.

use std::time;
use std::thread;
use std::io::{self, Seek, Write};

extern crate framebuffer;
pub use framebuffer::framebuffer::Framebuffer;

pub trait FramebufferExt {
    fn writer(&mut self, width: usize, height: usize) -> FbWriter;
    fn write(&mut self, offset: usize, data: &[u8]) -> io::Result<()>;
    fn write_loop<R, F>(
        &mut self,
        width: usize,
        height: usize,
        render_frame: F,
    ) -> Option<R>
        where
        F: FnMut(&mut FbWriter) -> Option<R>;
}

pub struct FbWriter<'a> {
    fb: &'a mut framebuffer::Framebuffer,
    offset: usize,
    width: usize,
    height: usize,
}

impl FramebufferExt for Framebuffer {
    fn writer(&mut self, width: usize, height: usize) -> FbWriter {
        let x = (self.var_screen_info.xres as usize - width) / 2;
        let y = (self.var_screen_info.yres as usize - height) * 4 / 5;
        let bytes_per_pixel = self.var_screen_info.bits_per_pixel as usize / 8;
        let offset = (y + self.var_screen_info.yoffset as usize)
            * self.fix_screen_info.line_length as usize
            + (x + self.var_screen_info.xoffset as usize) * bytes_per_pixel;
        FbWriter {
            fb: self,
            offset: offset,
            width: width * bytes_per_pixel,
            height: height,
        }
    }
    fn write_loop<R, F>(
        &mut self,
        width: usize,
        height: usize,
        mut render_frame: F,
    ) -> Option<R>
    where
        F: FnMut(&mut FbWriter) -> Option<R>,
    {
        let mut writer = self.writer(width, height);
        let dur = time::Duration::from_millis(1000 / 30);
        loop {
            let next = time::Instant::now() + dur;
            match render_frame(&mut writer) {
                Some(r) => return Some(r),
                None => (),
            };
            let now = time::Instant::now();
            if now < next {
                thread::sleep(next - now);
            }
        }
    }
    fn write(&mut self, offset: usize, data: &[u8]) -> io::Result<()> {
        self.device.seek(io::SeekFrom::Start(offset as u64))?;
        self.device.write_all(data)
    }

}

impl<'a> FbWriter<'a> {
    pub fn write(&mut self, frame: &[u8]) -> io::Result<()> {
        let mut offset = self.offset;
        let mut input = 0;
        for _ in 0..self.height {
            self.fb.write(offset, &frame[input..input + self.width])?;
            input += self.width;
            offset += self.fb.fix_screen_info.line_length as usize;
        }
        Ok(())
    }
}
