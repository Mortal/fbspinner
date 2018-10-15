import glob
import struct
import zlib

import numpy as np
from imageio import imread


def main():
    images = np.array(
        [flatten(imread(path)) for path in sorted(glob.glob("anim/frame*.png"))]
    )
    n, h, w, d = images.shape
    with open("anim.bin", "wb") as fp:
        fp.write(struct.pack("iiii", n, h, w, d))
        fp.write(zlib.compress(images.ravel().tobytes()))


def flatten(im):
    if im.ndim == 2:
        im = im[:, :, np.newaxis]
        im = im[:, :, [0, 0, 0]]
    height, width, bpp = im.shape
    assert bpp == 3
    im = im[:, :, ::-1]
    im = np.concatenate((im, np.zeros_like(im[:, :, :1])), axis=2)
    return im


if __name__ == "__main__":
    main()
