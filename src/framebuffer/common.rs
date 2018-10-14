#[cfg(target_pointer_width = "64")]
#[allow(dead_code)]
pub type NativeWidthType = u64;
#[cfg(target_pointer_width = "32")]
#[allow(dead_code)]
pub type NativeWidthType = u32;

pub const FBIOGET_VSCREENINFO: u64 = 0x4600;
pub const FBIOGET_FSCREENINFO: u64 = 0x4602;
// pub const FBIOPUT_VSCREENINFO: NativeWidthType = 0x4601;
// pub const FBIOGETCMAP: NativeWidthType = 0x4604;
// pub const FBIOPUTCMAP: NativeWidthType = 0x4605;
// pub const FBIOPAN_DISPLAY: NativeWidthType = 0x4606;
// pub const FBIO_CURSOR: NativeWidthType = 0x4608;
