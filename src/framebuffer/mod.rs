use libc::ioctl;

use std::io;
use std::io::{Seek, Write};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

mod common;
use framebuffer::common::{
    FBIOGET_FSCREENINFO, FBIOGET_VSCREENINFO,
};
mod screeninfo;
use framebuffer::screeninfo::{FixScreeninfo, VarScreeninfo};

/// Framebuffer struct containing the state (latest update marker etc.)
/// along with the var/fix screeninfo structs.
pub struct Framebuffer {
    pub device: File,
    pub var_screen_info: VarScreeninfo,
    pub fix_screen_info: FixScreeninfo,
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

pub struct FbWriter<'a> {
    fb: &'a mut Framebuffer,
    offset: usize,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn new(path_to_device: &str) -> Framebuffer {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_to_device)
            .unwrap();

        let fix_screen_info = Framebuffer::get_fix_screeninfo(&device);
        let var_screen_info = Framebuffer::get_var_screeninfo(&device);

        // Load the font
        Framebuffer {
            device,
            var_screen_info,
            fix_screen_info,
        }
    }

    fn get_fix_screeninfo(device: &File) -> FixScreeninfo {
        let mut info: FixScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO, &mut info) };
        if result != 0 {
            panic!("FBIOGET_FSCREENINFO failed");
        }
        info
    }

    fn get_var_screeninfo(device: &File) -> VarScreeninfo {
        let mut info: VarScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO, &mut info) };
        if result != 0 {
            panic!("FBIOGET_VSCREENINFO failed");
        }
        info
    }

    pub fn writer(&mut self, width: usize, height: usize) -> FbWriter {
        let x = (self.var_screen_info.xres as usize - width) / 2;
        let y = (self.var_screen_info.yres as usize - height) * 4 / 5;
        let bytes_per_pixel = self.var_screen_info.bits_per_pixel as usize / 8;
        let offset =
            (y + self.var_screen_info.yoffset as usize) *
            self.fix_screen_info.line_length as usize +
            (x + self.var_screen_info.xoffset as usize) *
            bytes_per_pixel;
        FbWriter {
            fb: self,
            offset: offset,
            width: width * bytes_per_pixel,
            height: height,
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
            self.fb.write(offset, &frame[input..input+self.width])?;
            input += self.width;
            offset += self.fb.fix_screen_info.line_length as usize;
        }
        Ok(())
    }
}
