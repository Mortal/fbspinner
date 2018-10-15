import cffi

ffibuilder = cffi.FFI()
ffibuilder.set_source(
    "convertlib",
    """
#include <linux/fb.h>
#include <sys/ioctl.h>

int getfbdims(int fbfd) {
    struct fb_var_screeninfo vinfo;
    if (ioctl(fbfd, FBIOGET_VSCREENINFO, &vinfo) == -1) {
        return -1;
    }
    return (vinfo.yres << 17) + (vinfo.xres << 2) + (vinfo.bits_per_pixel / 8 % 4);
};
""",
)
ffibuilder.cdef("int getfbdims(int fd);")


if __name__ == "__main__":
    ffibuilder.compile(verbose=True)
