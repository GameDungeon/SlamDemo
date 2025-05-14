use std::{f64::consts::PI, thread::{self, sleep}, time::Duration};

use server::{server_main, ServerMessage};
use tokio::{sync::broadcast, time::interval};
use tokio::sync::broadcast::Sender;
use image::{codecs::png::PngEncoder, EncodableLayout, GenericImageView, GrayImage, ImageDecoder, ImageEncoder, ImageReader, SubImage};

pub mod server;

struct Pose {
    x: f64,
    y: f64,
    rotation: f64
}

impl Pose {
    fn new(x: f64, y: f64, rotation: f64) -> Pose {
        Pose {
            x,
            y,
            rotation
        }
    }

    fn pos(&self) -> Point {
        Point {
            x: self.x as i32,
            y: self.y as i32
        }
    }
}

#[derive(Clone, Debug)]
struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

}

fn supercover_line(p0: Point, p1: Point, img: &GrayImage) -> Point {
    let dx = p1.x-p0.x;
    let dy = p1.y-p0.y;

    let nx = dx.abs();
    let ny = dy.abs();

    let sign_x = dx.signum();
    let sign_y = dy.signum();

    let mut p = Point::new(p0.x, p0.y);

    // let mut points = vec![p.clone()];

    let mut ix = 0;
    let mut iy = 0;

    while ix < nx || iy < ny {
        let decision = (1 + 2*ix) * ny - (1 + 2*iy) * nx;
        if decision == 0 {
            // next step is diagonal
            p.x += sign_x;
            p.y += sign_y;
            ix += 1;
            iy += 1;
        } else if decision < 0 {
            // next step is horizontal
            p.x += sign_x;
            ix += 1;
        } else {
            // next step is vertical
            p.y += sign_y;
            iy += 1;
        }

        // points.push(Point::new(p.x, p.y));

        if img.get_pixel(p.x as u32, p.y as u32)[0] == 0 {
            break;
        }
    }

    p
}

async fn robot_loop(sender: Sender<ServerMessage>) {
    let img = ImageReader::open("./map/map1.png").unwrap().decode().
        unwrap().into_luma8();

    

    let pose = Pose {
        x: 100.0,
        y: 100.0,
        rotation: 0.0
    };

    let mut ray_rot: f64 = 0.0;

    let ray_len: f64 = 100.0;

    let mut bytes: Vec<u8> = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);

    encoder.write_image(&img, 183, 183, image::ExtendedColorType::L8);

    loop {
        sender.send(ServerMessage::MapUpdate { x: 0, y: 0, img: Box::new(bytes.clone()) });
        sleep(Duration::from_secs_f64(10.0));
    }
    
    // loop {
    //     let x = ray_len * ray_rot.cos();
    //     let y = ray_len * ray_rot.sin();

    //     let ray_target = Point::new(x as i32, y as i32);

    //     dbg!(supercover_line(pose.pos(), ray_target, &img));

    //     ray_rot += 2.0 * PI / 360.0;

    //     if ray_rot >= 2.0 * PI {
    //         break;
    //     }
        
    //     sleep(Duration::from_secs_f64(1.0 / 360.0));
    // }
}

#[tokio::main]
async fn main() {
    env_logger::init();


    let (tx, _rx1) = broadcast::channel(16);

    tokio::spawn(server_main(tx.clone()));

    robot_loop(tx).await;
}
