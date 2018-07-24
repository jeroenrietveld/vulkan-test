use cgmath;
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector3};
use std;
use std::ops::Add;
use std::ops::Sub;
use winit;

pub struct Camera {
    position: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    world_up: Vector3<f32>,

    // TODO: Still needed
    pub projection: Matrix4<f32>,
    pub world: Matrix4<f32>,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    zoom: f64,
}

impl Camera {
    pub fn new() -> Camera {
        let projection = cgmath::perspective(
            cgmath::Rad(std::f32::consts::FRAC_PI_2),
            {
                1.5f32
                // let d = images[0].dimensions();
                // d[0] as f32 / d[1] as f32
            },
            0.01,
            100.0,
        );
        let world: Matrix4<f32> = <cgmath::Matrix4<f32> as cgmath::SquareMatrix>::identity();

        let position = Vector3::new(0.0, 0.0, 3.0);
        let world_up = Vector3::new(0.0, 1.0, 3.0);

        let yaw = -90.0;
        let pitch = 0.0;

        let x: f32 = Rad::cos(Rad::from(Deg(yaw))) * Rad::cos(Rad::from(Deg(pitch)));
        let y: f32 = Rad::sin(Rad::from(Deg(pitch)));
        let z: f32 = Rad::sin(Rad::from(Deg(yaw))) * Rad::cos(Rad::from(Deg(pitch)));
        let front = InnerSpace::normalize(Vector3::new(x, y, z));

        let right = InnerSpace::normalize(front.cross(world_up));
        let up = InnerSpace::normalize(right.cross(front));

        Camera {
            position,
            world_up,
            front,
            yaw,
            pitch,
            right,
            up,
            projection,
            world,
            movement_speed: 0.5,
            zoom: 45.0,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = Point3::new(self.position.x, self.position.y, self.position.z);
        let center = self.position.add(self.front);
        let center = Point3::new(center.x, center.y, center.z);
        let up = self.up;

        Matrix4::look_at(eye, center, up)
    }

    pub fn handle_input(&mut self, event: &winit::KeyboardInput, dt: f32) {
        //let pressed = event.state == winit::ElementState::Pressed;
        let velocity = self.movement_speed * dt;
        println!("Input!");

        match event.virtual_keycode {
            Some(winit::VirtualKeyCode::W) | Some(winit::VirtualKeyCode::Up) => {
                // Forward
                println!("Input! forward");
                self.position = self.position.add(self.front * velocity);
            }
            Some(winit::VirtualKeyCode::S) | Some(winit::VirtualKeyCode::Down) => {
                // Backward
                println!("Input! backward");
                self.position = self.position.sub(self.front * velocity);
            }
            Some(winit::VirtualKeyCode::A) | Some(winit::VirtualKeyCode::Left) => {
                // Left
                println!("Input! left");
                self.position = self.position.sub(self.right * velocity);
            }
            Some(winit::VirtualKeyCode::D) | Some(winit::VirtualKeyCode::Right) => {
                // Right
                println!("Input! right");
                self.position = self.position.add(self.right * velocity);
            }
            _ => (),
        };
    }
}
