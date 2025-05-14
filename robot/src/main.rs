#![no_main]
#![no_std]

extern crate alloc;

use core::{cell::RefCell, f64::consts::{PI, SQRT_2}, time::Duration};

use vexide::prelude::*;

use alloc::rc::Rc;

struct Robot {
    controller: Controller,

    imu: Rc<RefCell<InertialSensor>>,
    drivetrain: Rc<RefCell<Drive>>
}

struct Drive {
    front_left_motor: Motor,
    back_left_motor: Motor,
    front_right_motor:Motor,
    back_right_motor: Motor,
}

impl Compete for Robot {
    async fn driver(&mut self) {
        loop {
            let controller_state = self.controller.state().unwrap_or_default();

            let x = controller_state.left_stick.x();
            let y = controller_state.left_stick.y();

            let direction = y.atan2(x);
            let rotation = controller_state.right_stick.x();
            let mut speed = (x*x + y*y).sqrt();

            if speed == 0.0 {
                speed = f64::EPSILON
            }

            let p1 = -f64::cos(direction + PI / 4.0);
            let p2 = f64::sin(direction + PI / 4.0);

            let s = p1.abs().max(p2.abs()) / speed;

            let m_fl = (p2 / s) * (1.0 - rotation.abs()) + rotation;
            let m_fr = (p1 / s) * (1.0 - rotation.abs()) - rotation;
            let m_bl = (p1 / s) * (1.0 - rotation.abs()) + rotation;
            let m_br = (p2 / s) * (1.0 - rotation.abs()) - rotation;

            {
                let mut drive = self.drivetrain.borrow_mut();

                drive.front_left_motor.set_voltage(m_fl * Motor::V5_MAX_VOLTAGE).ok();
                drive.front_right_motor.set_voltage(m_fr * Motor::V5_MAX_VOLTAGE).ok();
                drive.back_left_motor.set_voltage(m_bl * Motor::V5_MAX_VOLTAGE).ok();
                drive.back_right_motor.set_voltage(m_br * Motor::V5_MAX_VOLTAGE).ok();
            }

            sleep(Controller::UPDATE_INTERVAL).await;
        }
    }
}

#[inline(always)]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * (PI / 180.0)
}

async fn odom(drivetrain: Rc<RefCell<Drive>>, imu: Rc<RefCell<InertialSensor>>) {
    const WHEEL_CIRC: f64 = 2.0 * PI;
    
    let mut last_fl = 0.0;
    let mut last_fr = 0.0;
    let mut last_bl = 0.0;
    let mut last_br = 0.0;

    let mut rot;
    let mut x = 0.0;
    let mut y = 0.0;

    loop {
        {
            let drive = drivetrain.borrow_mut();

            rot = imu.borrow().heading().unwrap_or_default();

            let fl_pos = drive.front_left_motor.position().unwrap_or_default().as_revolutions();
            let fr_pos = -drive.front_right_motor.position().unwrap_or_default().as_revolutions();
            let bl_pos = drive.back_left_motor.position().unwrap_or_default().as_revolutions();
            let br_pos = -drive.back_right_motor.position().unwrap_or_default().as_revolutions();

            let fl_d = fl_pos - last_fl;
            let fr_d = fr_pos - last_fr;
            let bl_d = bl_pos - last_bl;
            let br_d = br_pos - last_br;

            let delta_x = (fl_d + fr_d - bl_d - br_d) * (1.0 / SQRT_2) * WHEEL_CIRC;
            let delta_y = (fl_d - fr_d + bl_d - br_d) * (1.0 / SQRT_2) * WHEEL_CIRC;

            let rad_rot = deg_to_rad(rot * -1.0);

            let delta_global_x = delta_x * rad_rot.cos() - delta_y * rad_rot.sin();
            let delta_global_y = delta_x * rad_rot.sin() + delta_y * rad_rot.cos();

            x += delta_global_x;
            y += delta_global_y;

            println!("odom {x} {y} {rot}");

            last_fl = fl_pos;
            last_fr = fr_pos;
            last_bl = bl_pos;
            last_br = br_pos;
        }

        // sleep(InertialSensor::MIN_DATA_INTERVAL).await;
        sleep(Duration::from_millis(50)).await;
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut imu = InertialSensor::new(peripherals.port_13);

    imu.calibrate().await.ok();
    
    let robot = Robot {
        controller: peripherals.primary_controller,

        imu: Rc::new(RefCell::new(imu)),
        
        drivetrain: Rc::new(RefCell::new(Drive {
            front_left_motor: Motor::new(peripherals.port_21, Gearset::Green, Direction::Forward),
            front_right_motor:Motor::new(peripherals.port_20, Gearset::Green, Direction::Reverse),
            back_left_motor: Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
            back_right_motor: Motor::new(peripherals.port_11, Gearset::Green, Direction::Reverse)
        }))
    };

    spawn(odom(robot.drivetrain.clone(), robot.imu.clone())).detach();

    robot.compete().await;
}
