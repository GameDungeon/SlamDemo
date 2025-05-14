use gs_rs::parser::model::{FactorGraphModel, Vertex};
use json::JsonValue;
use plotters::prelude::*;

use image::{self, ImageBuffer, Rgb, RgbImage};
use plotters::style::RGBColor;

use core::{f32, f64};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;

use icp_2d::{ICPPoint, Icp};
use nalgebra::Point2;
use nalgebra::{self as na, Point1};

use core::f64::consts;

const OUT_FILE_NAME: &str = "slam.gif";
const IN_DATA: &str = "./preprocess_data/data.log";

const START_X: f64 = 0.0;
const START_Y: f64 = 0.0;
const MAP_SIZE: f64 = 600.0;

const MAX_LIDAR_RANGE: f64 = 10.0;

struct SLAM {
    scans: Vec<Vec<Point2<f64>>>,
    map: RgbImage,

    width: f64,
    height: f64,

    graph: FactorGraphModel,
}

impl SLAM {
    fn new(width: u32, height: u32) -> SLAM {
        let map: RgbImage = ImageBuffer::from_pixel(width, height, Rgb([200, 200, 200]));

        let pose = Vertex {
            id: 0,
            vertex_type: String::from("Vehicle2D"),
            content: vec![START_X, START_Y, 0.0],
        };

        let mut fixed_vertices = BTreeSet::new();
        fixed_vertices.insert(0);

        let graph = FactorGraphModel {
            vertices: vec![pose],
            edges: Vec::new(),
            fixed_vertices,
        };

        SLAM {
            scans: vec![Vec::new()],
            map,

            width: width as f64,
            height: height as f64,

            graph,
        }
    }

    fn add_measurement(&mut self, scan: JsonValue) {
        let x = scan["x"].as_f64().unwrap() + START_X;
        let y = scan["y"].as_f64().unwrap() + START_Y;
        let theta = scan["theta"].as_f64().unwrap();

        let pose = Vertex {
            id: self.graph.vertices.len(),
            vertex_type: String::from("Vehicle2D"),
            content: vec![x, y, theta],
        };

        self.graph.vertices.push(pose);

        let mut points = Vec::new();

        let mut theta_mut = theta - consts::FRAC_PI_2;

        for ray in scan["range"].members() {
            let rangef = ray.as_f64().unwrap();

            if rangef > MAX_LIDAR_RANGE {
                continue;
            }

            let x = rangef * theta_mut.cos();
            let y = rangef * theta_mut.sin();

            theta_mut += consts::PI / 180.0;

            points.push(Point2::new(x, y))
        }

        self.scans.push(points);
    }

    fn visualize(&mut self) {
        let wratio = self.width / MAP_SIZE;
        let hratio = self.height / MAP_SIZE;

        for index in 0..self.graph.vertices.len() {
            let pose = &self.graph.vertices[index];

            let x = pose.content[0];
            let y = pose.content[1];
            // let r = pose.content[2];

            let pos = Point2::new(x, y);

            for point in &self.scans[index] {
                let pixels = SLAM::supercover_line(&pos, point);

                for pixel in &pixels {
                    self.map.put_pixel(
                        ((pos.x + pixel.x) * wratio + self.width / 2.0) as u32,
                        (self.height - ((pos.y + pixel.y) * hratio + self.height / 2.0)) as u32,
                        Rgb([255, 255, 255]),
                    );
                }

                self.map.put_pixel(
                    ((pos.x + self.width / 2.0) * wratio) as u32,
                    ((self.height - (pos.y + self.height / 2.0)) * hratio) as u32,
                    Rgb([0, 0, 0]),
                );
            }
        }
    }

    fn supercover_line(p0: &Point2<f64>, p1: &Point2<f64>) -> Vec<Point2<f64>> {
        let dx = (p1.x - p0.x) as i32;
        let dy = (p1.y - p0.y) as i32;

        let nx = dx.abs();
        let ny = dy.abs();

        let sign_x = dx.signum();
        let sign_y = dy.signum();

        let mut p = Point2::new(p0.x, p0.y);

        let mut points = vec![p];

        let mut ix = 0;
        let mut iy = 0;

        while ix < nx || iy < ny {
            let decision = (1 + 2 * ix) * ny - (1 + 2 * iy) * nx;
            if decision == 0 {
                // next step is diagonal
                p.x += sign_x as f64;
                p.y += sign_y as f64;
                ix += 1;
                iy += 1;
            } else if decision < 0 {
                // next step is horizontal
                p.x += sign_x as f64;
                ix += 1;
            } else {
                // next step is vertical
                p.y += sign_y as f64;
                iy += 1;
            }

            points.push(Point2::new(p.x as f64, p.y as f64));
        }

        points
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(IN_DATA).expect("Should have been able to read the file");

    // let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();
    let root = BitMapBackend::gif(OUT_FILE_NAME, (1000, 1000), 200)
        .unwrap()
        .into_drawing_area();

    let mut chart = ChartBuilder::on(&root).margin(10).build_cartesian_2d(
        -(MAP_SIZE / 2.0)..(MAP_SIZE / 2.0),
        -(MAP_SIZE / 2.0)..(MAP_SIZE / 2.0),
    )?;

    let pixel_range = chart.plotting_area().get_pixel_range();
    let p_width = (pixel_range.0.end - pixel_range.0.start) as u32;
    let p_height = (pixel_range.1.end - pixel_range.1.start) as u32;

    let mut slam = SLAM::new(p_width, p_height);

    let mut i = 0;
    for scan in contents.split("\n") {
        if scan.is_empty() {
            break;
        }

        slam.add_measurement(json::parse(scan).unwrap());
        i += 1;

        if i % 1000 != 0 {
            continue;
        }

        dbg!(i);

        slam.visualize();

        root.fill(&WHITE)?;

        for pixel in slam.map.enumerate_pixels() {
            let pd = pixel.2 .0;
            root.draw_pixel(
                (pixel.0 as i32, pixel.1 as i32),
                &RGBColor(pd[0], pd[1], pd[2]),
            )
            .unwrap();
        }

        chart.draw_series(LineSeries::new(
            slam.graph
                .vertices
                .iter()
                .map(|pose| (pose.content[0], pose.content[1])),
            &BLUE,
        ))?;

        let current_pos = &slam.graph.vertices[slam.graph.vertices.len() - 1].content;

        chart.draw_series(std::iter::once(Circle::new(
            (current_pos[0], current_pos[1]),
            2,
            RED.filled(),
        )))?;

        root.present().unwrap();
    }

    root.present().unwrap();

    Ok(())
}
