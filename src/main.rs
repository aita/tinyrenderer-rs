use clap::Parser;
use glam::{IVec2, Vec2, Vec3};
use image::{imageops, ImageBuffer, Rgba, RgbaImage};
use rand::{thread_rng, Rng};

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

fn barycentric(a: Vec3, b: Vec3, c: Vec3, p: Vec3) -> Vec3 {
    let mut s = [Vec3::ZERO; 2];
    for i in 0..2 {
        s[i][0] = c[i] - a[i];
        s[i][1] = b[i] - a[i];
        s[i][2] = a[i] - p[i];
    }
    let u = s[0].cross(s[1]);
    if u[2].abs() > 1e-2 {
        Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
    } else {
        Vec3::new(-1.0, 1.0, 1.0)
    }
}

fn triangle(pts: [Vec3; 3], zbuffer: &mut [f32], image: &mut RgbaImage, color: Rgba<u8>) {
    let mut bboxmin = Vec2::new(f32::MAX, f32::MAX);
    let mut bboxmax = Vec2::new(f32::MIN, f32::MIN);
    let clamp = Vec2::new(image.width() as f32 - 1.0, image.height() as f32 - 1.0);
    for i in 0..3 {
        for j in 0..2 {
            bboxmin[j] = 0.0f32.max(bboxmin[j].min(pts[i][j]));
            bboxmax[j] = clamp[j].min(bboxmax[j].max(pts[i][j]));
        }
    }
    let mut p = Vec3::ZERO;
    p.x = bboxmin.x;
    while p.x <= bboxmax.x {
        p.y = bboxmin.y;
        while p.y <= bboxmax.y {
            let bc_screen = barycentric(pts[0], pts[1], pts[2], p);
            if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                p.y += 1.0;
                continue;
            }
            p.z = 0.0;
            for i in 0..3 {
                p.z += pts[i][2] * bc_screen[i];
            }
            let index = (p.x + p.y * image.width() as f32) as usize;
            if zbuffer[index] < p.z {
                zbuffer[index] = p.z;
                put_pixel(image, p.x as u32, p.y as u32, color);
            }
            p.y += 1.0;
        }
        p.x += 1.0;
    }
}

fn world2screen(v: Vec3, width: f32, height: f32) -> Vec3 {
    Vec3::new(
        ((v.x + 1.0) * width / 2.0 + 0.5).floor(),
        ((v.y + 1.0) * height / 2.0 + 0.5).floor(),
        v.z,
    )
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
    let mut zbuffer = vec![f32::MIN; (args.width * args.height) as usize];

    let mut rng = thread_rng();

    let width = args.width as f32;
    let height = args.height as f32;
    for i in 0..model.nfaces() {
        let face = model.face(i);
        let mut world_coords = [Vec3::ZERO; 3];
        for i in 0..3 {
            world_coords[i] = world2screen(model.vert(face[i]), width, height);
        }
        triangle(
            world_coords,
            &mut zbuffer,
            &mut image,
            Rgba([
                (rng.gen_range(0.0..1.0) * 255.0) as u8,
                (rng.gen_range(0.0..1.0) * 255.0) as u8,
                (rng.gen_range(0.0..1.0) * 255.0) as u8,
                255,
            ]),
        );
    }

    imageops::flip_vertical_in_place(&mut image);
    image.save(args.output).unwrap();
}
