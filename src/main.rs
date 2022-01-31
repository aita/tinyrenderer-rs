use clap::Parser;
use image::{imageops, ImageBuffer, Rgba, RgbaImage};

mod model;

fn line(x0: i32, y0: i32, x1: i32, y1: i32, image: &mut RgbaImage, color: &Rgba<u8>) {
    let mut x0 = x0;
    let mut x1 = x1;
    let mut y0 = y0;
    let mut y1 = y1;
    let mut steep = false;
    if (x0 - x1).abs() < (y0 - y1).abs() {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    let dx = x1 - x0;
    let dy = y1 - y0;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = y0;
    for x in x0..=x1 {
        if steep {
            put_pixel(image, y as u32, x as u32, *color);
        } else {
            put_pixel(image, x as u32, y as u32, *color);
        }
        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
}

fn put_pixel(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= image.width() || y >= image.height() {
        return;
    }
    image.put_pixel(x, y, color);
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 800)]
    width: u32,

    #[clap(short, long, default_value_t = 800)]
    height: u32,

    #[clap(short, long, default_value = "obj/african_head/african_head.obj")]
    model: String,

    #[clap(short, long, default_value = "output.png")]
    output: String,
}

fn main() {
    const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

    let args = Args::parse();
    let model = model::Model::load_obj(&args.model);
    let mut image: RgbaImage = ImageBuffer::new(args.width, args.height);
    let width = args.width as f32;
    let height = args.height as f32;
    for i in 0..model.nfaces() {
        let face = model.face(i);
        for j in 0..3 {
            let v0 = model.vert(face[j]);
            let v1 = model.vert(face[(j + 1) % 3]);
            let x0 = (v0.x + 1.0) * width / 2.0;
            let y0 = (v0.y + 1.0) * height / 2.0;
            let x1 = (v1.x + 1.0) * width / 2.0;
            let y1 = (v1.y + 1.0) * height / 2.0;
            line(
                x0 as i32, y0 as i32, x1 as i32, y1 as i32, &mut image, &WHITE,
            );
        }
    }

    imageops::flip_vertical_in_place(&mut image);
    image.save(args.output).unwrap();
}
