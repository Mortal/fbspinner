//!Simple Linux framebuffer abstraction.

extern crate libc;

use libc::ioctl;

use std::error::Error;
use std::fmt;
use std::io;
use std::fs::{File, OpenOptions};
use std::io::{Seek, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;

///Bitfield which is a part of VarScreeninfo.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Bitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

///Struct as defined in /usr/include/linux/fb.h
#[repr(C)]
#[derive(Clone, Debug)]
pub struct VarScreeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: Bitfield,
    pub green: Bitfield,
    pub blue: Bitfield,
    pub transp: Bitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

///Struct as defined in /usr/include/linux/fb.h Note: type is a keyword in Rust and therefore has been
///changed to fb_type.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FixScreeninfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: u32,
    pub fb_type: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

impl ::std::default::Default for Bitfield {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl ::std::default::Default for VarScreeninfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl ::std::default::Default for FixScreeninfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

///Kind of errors that can occur when dealing with the Framebuffer.
#[derive(Debug)]
pub enum FramebufferErrorKind {
    IoctlFailed,
    IoError,
}

#[derive(Debug)]
pub struct FramebufferError {
    pub kind: FramebufferErrorKind,
    pub details: String,
}

impl FramebufferError {
    fn new(kind: FramebufferErrorKind, details: &str) -> FramebufferError {
        FramebufferError {
            kind,
            details: String::from(details),
        }
    }
}

impl Error for FramebufferError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl From<io::Error> for FramebufferError {
    fn from(err: io::Error) -> FramebufferError {
        FramebufferError::new(FramebufferErrorKind::IoError, err.description())
    }
}

///Struct that should be used to work with the framebuffer.
#[derive(Debug)]
pub struct Framebuffer {
    pub device: File,
    pub var_screen_info: VarScreeninfo,
    pub fix_screen_info: FixScreeninfo,
}

pub struct FbWriter<'a> {
    fb: &'a mut Framebuffer,
    offset: usize,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn new<P: AsRef<Path>>(path_to_device: P) -> Result<Framebuffer, FramebufferError> {
        let device = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path_to_device)?;

        let var_screen_info = Framebuffer::get_var_screeninfo(&device)?;
        let fix_screen_info = Framebuffer::get_fix_screeninfo(&device)?;

        Ok(Framebuffer {
            device,
            var_screen_info,
            fix_screen_info,
        })
    }

    ///Prepares writing a sprite of the given size.
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

    ///Creates a FixScreeninfo struct and fills it using ioctl.
    pub fn get_fix_screeninfo(device: &File) -> Result<FixScreeninfo, FramebufferError> {
        let mut info: FixScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                "Ioctl returned -1",
            )),
            _ => Ok(info),
        }
    }

    ///Creates a VarScreeninfo struct and fills it using ioctl.
    pub fn get_var_screeninfo(device: &File) -> Result<VarScreeninfo, FramebufferError> {
        let mut info: VarScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                "Ioctl returned -1",
            )),
            _ => Ok(info),
        }
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
