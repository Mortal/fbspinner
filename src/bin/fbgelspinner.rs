extern crate bmp;
extern crate fbspinner;
extern crate flate2;
extern crate gel;

use gel::obj::Obj;
use gel::{ZBufferedTarget, Viewport, Vertex, Pixels, TextureShader};
use fbspinner::framebuffer;
use std::env;

struct BmpPixels {
    image: bmp::Image,
}

impl Pixels for BmpPixels {
    fn get_pixel(&self, xx: usize, yy: usize) -> (u8, u8, u8) {
        let p = self.image.get_pixel(xx as u32, yy as u32);
        (p.r, p.g, p.b)
    }
}

fn main() {
    let model = {
        let mut args = env::args();
        let program_name = args.next().unwrap_or("gel".to_owned());
        if let Some(arg) = args.next() {
            if arg.starts_with("-") {
                println!("Usage: {} [model_name]", program_name);
                return;
            }
            arg
        } else {
            "salesman".to_owned()
        }
    };

    let obj_filename = format!("model/{}.obj", model);
    let obj = match Obj::load(&obj_filename) {
        Ok(obj) => obj,
        Err(e) => {
            println!("Could not read {}: {}", obj_filename, e);
            return;
        }
    };
    let bmp_filename = format!("model/{}.bmp", model);
    let image = match bmp::open(&bmp_filename) {
        Ok(bmp) => bmp,
        Err(e) => {
            println!("Could not read {}: {}", &bmp_filename, e);
            return;
        }
    };
    let (image_width, image_height) = (image.get_width(), image.get_height());
    let shader = TextureShader {
        pixels: BmpPixels { image },
        width: image_width,
        height: image_height,
    };

    let mut fb = match framebuffer::Framebuffer::new("/dev/fb0") {
        Ok(fb) => fb,
        Err(e) => {
            println!("Could not open /dev/fb0 ({})", e);
            return;
        }
    };

    let xres = 400;
    let yres = 300;

    let mut zbuff = Vec::new();
    zbuff.resize((xres * yres) as usize, 0f32);
    let mut pixel = Vec::new();
    pixel.resize((xres * yres * 4) as usize, 0u8);

    let mut i = 0;
    fb.write_loop(xres, yres, |writer| {

        let rot = i as f32 * 0.04;
        let eye_dir = Vertex {
            x: rot.sin(),
            y: 0.0,
            z: rot.cos(),
        };

        let z = (eye_dir.clone() - Vertex::center()).unit();
        let x = (Vertex::upward().cross(z.clone())).unit();
        let y = z.clone().cross(x.clone());

        {
            let mut target = ZBufferedTarget::new_row_major(xres, yres, &mut pixel, &mut zbuff);
            target.reset();

            let mut viewport = Viewport {
                target: target,
                x: x,
                y: y,
                z: z,
                eye: eye_dir.clone(),
            };

            obj.draw_shaded(&mut viewport, &shader);
        }

        writer
            .write(&pixel)
            .unwrap();
        i += 1;
        None as Option<()>
    });
}
