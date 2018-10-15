extern crate flate2;
extern crate libc;

mod framebuffer;

use flate2::read::ZlibDecoder;
use std::io::Read;
use std::{fmt, fs, io, result, thread, time};

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
        ((buf[3] as u32) << 24)
            + ((buf[2] as u32) << 16)
            + ((buf[1] as u32) << 8)
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

fn main() {
    let animdata = match fs::File::open("anim.bin") {
        Ok(f) => f,
        Err(e) => {
            println!("Could not open anim.bin ({})", e);
            return;
        }
    };

    let (nframes, height, width, frames, frame_size) = match read_frames(animdata) {
        Ok(x) => x,
        Err(e) => {
            println!("anim.bin is in the wrong format ({})", e);
            return;
        }
    };

    let mut fb = match framebuffer::Framebuffer::new("/dev/fb0") {
        Ok(fb) => fb,
        Err(e) => {
            println!("Could not open /dev/fb0 ({})", e);
            return;
        }
    };

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
