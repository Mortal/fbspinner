# 199-82
# 291-174
# 30 fps

from imageio import imread, imwrite
import numpy as np


def main():
    filenames = ["boot2/boot%03d.png" % n for n in range(82, 199)]
    images = [imread(f)[134 : 134 + 107, 233 : 233 + 143, ...] for f in filenames]
    any_x = np.any(images, axis=(0, 1, 3)).nonzero()[0]
    any_y = np.any(images, axis=(0, 2, 3)).nonzero()[0]
    x0, x1 = any_x[0], any_x[-1] + 1
    y0, y1 = any_y[0], any_y[-1] + 1
    images = [im[y0:y1, x0:x1, ...] for im in images]
    for i, im in enumerate(images):
        imwrite("anim/frame%03d.png" % i, im)


if __name__ == "__main__":
    main()
