extern crate flate2;
extern crate libc;

mod framebuffer;

use flate2::read::ZlibDecoder;
use std::io::Read;
use std::{fmt, fs, io, result, thread, time};

#[derive(Debug)]
pub enum ErrorKind {
    CompressionUnknown(u32),
    Io(io::Error),
    UnexpectedEof,
    VersionIsZero,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

pub type Result<T> = result::Result<T, Error>;

impl Into<Error> for ErrorKind {
    fn into(self) -> Error {
        Error { kind: self }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            ErrorKind::UnexpectedEof.into()
        } else {
            ErrorKind::Io(e).into()
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::CompressionUnknown(c) => write!(f, "Unknown compression {}", c),
            ErrorKind::Io(ref e) => write!(f, "{}", e),
            ErrorKind::UnexpectedEof => write!(f, "Unexpected end-of-file"),
            ErrorKind::VersionIsZero => write!(f, "Version is 0"),
        }
    }
}

fn read_u32<R: io::Read>(r: &mut R) -> Result<u32> {
    let buf = &mut [0, 0, 0, 0];
    r.read_exact(buf)?;
    Ok(
        ((buf[3] as u32) << 24)
            + ((buf[2] as u32) << 16)
            + ((buf[1] as u32) << 8)
            + (buf[0] as u32),
    )
}

fn main() {
    let mut animdata = fs::File::open("anim.bin").unwrap();
    let nframes = read_u32(&mut animdata).unwrap() as usize;
    let height = read_u32(&mut animdata).unwrap() as usize;
    let width = read_u32(&mut animdata).unwrap() as usize;
    let bpp = read_u32(&mut animdata).unwrap() as usize;
    let frame_size = height * width * bpp;
    let mut frames = vec![0; nframes * frame_size];
    let mut decoder = ZlibDecoder::new(animdata);
    decoder.read_exact(&mut frames).unwrap();
    assert_eq!(0, decoder.read(&mut [0]).unwrap());

    let mut fb = framebuffer::Framebuffer::new("/dev/fb0").unwrap();
    let mut writer = fb.writer(width, height);
    let dur = time::Duration::from_millis(1000 / 30);
    loop {
        for i in 0..nframes {
            writer
                .write(&frames[i * frame_size..(i + 1) * frame_size])
                .unwrap();
            thread::sleep(dur);
        }
    }
}
