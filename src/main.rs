use clap::Parser;
use glam::{IVec2, Vec3};
use image::{imageops, ImageBuffer, Rgba, RgbaImage};

mod model;

fn put_pixel(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= image.width() || y >= image.height() {
        return;
    }
    image.put_pixel(x, y, color);
}

fn line(t0: IVec2, t1: IVec2, image: &mut RgbaImage, color: Rgba<u8>) {
    let mut x0 = t0[0];
    let mut x1 = t1[0];
    let mut y0 = t0[1];
    let mut y1 = t1[1];
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
            put_pixel(image, y as u32, x as u32, color);
        } else {
            put_pixel(image, x as u32, y as u32, color);
        }
        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
}

fn triangle(t0: IVec2, t1: IVec2, t2: IVec2, image: &mut RgbaImage, color: Rgba<u8>) {
    if t0.y == t1.y && t0.y == t2.y {
        return;
    }
    let mut t0 = t0;
    let mut t1 = t1;
    let mut t2 = t2;
    if t0.y > t1.y {
        std::mem::swap(&mut t0, &mut t1);
    }
    if t0.y > t2.y {
        std::mem::swap(&mut t0, &mut t2);
    }
    if t1.y > t2.y {
        std::mem::swap(&mut t1, &mut t2);
    }
    let total_height = t2.y - t0.y;
    for i in 0..total_height {
        let second_half = i > t1.y - t0.y || t1.y == t0.y;
        let segment_height = if second_half {
            t2.y - t1.y
        } else {
            t1.y - t0.y
        };
        let alpha = i as f32 / total_height as f32;
        let beta = (i - if second_half { t1.y - t0.y } else { 0 }) as f32 / segment_height as f32;
        let s0 = t0.as_vec2();
        let s1 = t1.as_vec2();
        let s2 = t2.as_vec2();
        let mut a = s0 + (s2 - s0) * alpha;
        let mut b = if second_half {
            s1 + (s2 - s1) * beta
        } else {
            s0 + (s1 - s0) * beta
        };
        if a.x > b.x {
            std::mem::swap(&mut a, &mut b);
        }
        let a = a.as_ivec2();
        let b = b.as_ivec2();
        for j in a.x..=b.x {
            put_pixel(image, j as u32, (t0.y + i) as u32, color);
        }
    }
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
    let args = Args::parse();
    let model = model::Model::load_obj(&args.model);
    let mut image: RgbaImage = ImageBuffer::new(args.width, args.height);
    let light_dir = Vec3::new(0.0, 0.0, -1.0);

    let width = args.width as f32;
    let height = args.height as f32;
    for i in 0..model.nfaces() {
        let face = model.face(i);
        let mut screen_coords = [IVec2::ZERO; 3];
        let mut world_coords = [Vec3::ZERO; 3];
        for j in 0..3 {
            let v = model.vert(face[j]);
            screen_coords[j] = IVec2::new(
                ((v.x + 1.0) * width / 2.0) as i32,
                ((v.y + 1.0) * height / 2.0) as i32,
            );
            world_coords[j] = v;
        }
        let n = (world_coords[2] - world_coords[0])
            .cross(world_coords[1] - world_coords[0])
            .normalize();
        let intensity = n.dot(light_dir);
        if intensity > 0.0 {
            triangle(
                screen_coords[0],
                screen_coords[1],
                screen_coords[2],
                &mut image,
                Rgba([
                    (intensity * 255.0) as u8,
                    (intensity * 255.0) as u8,
                    (intensity * 255.0) as u8,
                    255,
                ]),
            );
        }
    }

    imageops::flip_vertical_in_place(&mut image);
    image.save(args.output).unwrap();
}
