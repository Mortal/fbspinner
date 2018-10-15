import argparse
import os
import re

import numpy as np
from imageio import imread, imwrite

from convertlib import lib as convertlib


def parse_geometry(s):
    mo = re.match(r"^(\d+)x(\d+)$", s)
    if not mo:
        raise ValueError(s)
    return int(mo.group(1)), int(mo.group(2))


def is_fb(path):
    return path.startswith("/dev/fb") or path.endswith((".fb", ".raw"))


parser = argparse.ArgumentParser()
parser.add_argument("--geometry", type=parse_geometry)
parser.add_argument("input")
parser.add_argument("output")


def main():
    args = parser.parse_args()
    direction = is_fb(args.input), is_fb(args.output)
    if direction == (True, False):
        convert_fb_to_image(args)
    elif direction == (False, True):
        convert_image_to_fb(args)
    else:
        parser.error(
            "Exactly one argument must be framebuffer (/dev/fb* or *.fb or *.raw)"
        )


def get_dimensions(fp, geometry):
    sz = os.fstat(fp.fileno()).st_size
    if geometry and sz:
        width, height = geometry
        bpp, zero = divmod(sz, width * height)
        if zero != 0:
            raise Exception("Size %s not equal to %s*%s*%s" % (sz, width, height, bpp))
    else:
        if geometry:
            print("Ignoring --geometry")
        pixels, bpp = divmod(convertlib.getfbdims(fp.fileno()), 4)
        if bpp == 0:
            bpp = 4
        height, width = divmod(pixels, 2 ** 15)
    return width, height, bpp


def convert_fb_to_image(args):
    with open(args.input, "rb") as fp:
        width, height, bpp = get_dimensions(fp, args.geometry)
        pixels = np.fromfile(fp, np.uint8, width * height * bpp).reshape(
            (height, width, bpp)
        )
    if bpp == 4:
        pixels = pixels[:, :, :3]
    if pixels.shape[-1] == 3:
        pixels = pixels[:, :, ::-1]
    print(pixels.shape)
    imwrite(args.output, pixels)


def convert_image_to_fb(args):
    im = imread(args.input)
    input_height, input_width, input_bpp = im.shape
    with open(args.output, "r+b") as fp:
        width, height, bpp = get_dimensions(fp, args.geometry)
        if (width, height) != (input_width, input_height):
            parser.error(
                "image size %sx%s does not match fb size %sx%s"
                % (input_width, input_height, width, height)
            )
        if (bpp, input_bpp) == (4, 3):
            # Add zeros to input
            im = np.concatenate((im, np.zeros_like(im[:, :, :1])), axis=2)
        elif (bpp, input_bpp) == (3, 4):
            # Remove alpha channel
            im = im[:, :, :3]
        elif bpp != input_bpp:
            parser.error(
                "Don't know how to convert image depth %s to fb depth %s"
                % (input_bpp, bpp)
            )
        assert im.shape[2] == bpp

        # RGB->BGR
        if bpp == 3:
            im = im[:, :, ::-1]
        elif bpp == 4:
            im = im[:, :, [2, 1, 0, 3]]

        b = im.tobytes()
        assert len(b) == width * height * bpp
        fp.write(b)


if __name__ == "__main__":
    main()
