extern crate fbspinner;
extern crate flate2;

use fbspinner::framebuffer::{Framebuffer, FramebufferExt};
use flate2::read::ZlibDecoder;
use std::io::Read;
use std::{fmt, fs, io, process, result};

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    UnexpectedEof,
    ExpectedEof,
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
            ErrorKind::Io(ref e) => write!(f, "{}", e),
            ErrorKind::UnexpectedEof => write!(f, "Unexpected end-of-file"),
            ErrorKind::ExpectedEof => write!(f, "Expected end-of-file"),
        }
    }
}

fn read_u32<R: io::Read>(r: &mut R) -> Result<u32> {
    let buf = &mut [0, 0, 0, 0];
    r.read_exact(buf)?;
    Ok(
        ((buf[3] as u32) << 24) + ((buf[2] as u32) << 16) + ((buf[1] as u32) << 8)
            + (buf[0] as u32),
    )
}

fn read_frames(mut animdata: fs::File) -> Result<(usize, usize, usize, Vec<u8>, usize)> {
    let nframes = read_u32(&mut animdata)? as usize;
    let height = read_u32(&mut animdata)? as usize;
    let width = read_u32(&mut animdata)? as usize;
    let bpp = read_u32(&mut animdata)? as usize;
    let frame_size = height * width * bpp;
    let mut frames = vec![0; nframes * frame_size];
    let mut decoder = ZlibDecoder::new(animdata);
    decoder.read_exact(&mut frames)?;
    if decoder.read(&mut [0])? != 0 {
        return Err(ErrorKind::ExpectedEof.into());
    }
    Ok((nframes, height, width, frames, frame_size))
}

macro_rules! unwrap_or_exit {
    ( $x:expr, $msg:expr ) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                eprintln!($msg, e);
                process::exit(1);
            }
        }
    }
}

fn main() {
    let animdata = unwrap_or_exit!(fs::File::open("anim.bin"), "Could not open anim.bin ({})");

    let (nframes, height, width, frames, frame_size) = unwrap_or_exit!(
        read_frames(animdata),
        "anim.bin is in the wrong format ({})"
    );

    let mut fb = unwrap_or_exit!(
        Framebuffer::new("/dev/fb0"),
        "Could not open /dev/fb0 ({})"
    );

    let mut i = 0;
    fb.write_loop(width, height, |writer| {
        writer
            .write(&frames[i * frame_size..(i + 1) * frame_size])
            .unwrap();
        i += 1;
        if i == nframes {
            i = 0;
        }
        None as Option<()>
    });
}
