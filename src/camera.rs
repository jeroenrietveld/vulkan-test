use cgmath;
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector3, Vector4};
use std;
use std::ops::Add;
use std::ops::Sub;
use winit;

pub struct Camera {
    pub projection: Matrix4<f32>,
    pub world: Matrix4<f32>,
    view: Matrix4<f32>,
    model: Matrix4<f32>,
    clip: Matrix4<f32>,
    pub mvp: Matrix4<f32>,
    // TODO: find clean way
    pub zeroes: Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Camera {
        // let projection = cgmath::perspective(Rad(45.0), 1.0, 0.1, 100.0);
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
        let view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(0.0, 0.0, 5.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        );
        let world: Matrix4<f32> = <cgmath::Matrix4<f32> as cgmath::SquareMatrix>::identity().into();
        let mvp = projection * view * world;

        // let projection = cgmath::perspective(Rad::from(Deg(45.0)), 1.0, 0.1, 100.0);
        // let view = Matrix4::look_at(
        //     Point3::new(-5.0f32, 3.0f32, -10.0f32),
        //     Point3::new(0.0f32, 0.0f32, 0.0f32),
        //     Vector3::new(0.0f32, -1.0f32, 0.0f32),
        // );
        // :let view = Matrix4::look_at(
        //     Point3::new(0.0f32, 0.0f32, 0.0f32),
        //     Point3::new(0.0f32, 0.0f32, 0.0f32),
        //     Vector3::new(0.0f32, 0.0f32, 0.0f32),
        // );
        let vector_ones = Vector4::new(1.0f32, 1.0f32, 1.0f32, 1.0f32);
        let model = Matrix4::from_cols(vector_ones, vector_ones, vector_ones, vector_ones);
        let vector_zeroes = Vector4::new(0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let zeroes = Matrix4::from_cols(vector_zeroes, vector_zeroes, vector_zeroes, vector_zeroes);
        let clip = Matrix4::new(
            1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, -1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
            0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.5f32, 1.0f32,
        );
        // let mvp = clip * projection * view * model;
        println!(
            "projection: {:?}\n view {:?}\n model {:?}\n clip {:?}\n",
            projection, view, model, clip
        );

        Camera {
            projection,
            world,
            view,
            model,
            clip,
            mvp,
            zeroes,
        }
    }
}

pub struct Camera2 {
    position: cgmath::Vector3<f32>,
    front: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,
    world_up: cgmath::Vector3<f32>,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    zoom: f64,
}

impl Camera2 {
    pub fn new() -> Camera2 {
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

        Camera2 {
            position,
            world_up,
            front,
            yaw,
            pitch,
            right,
            up,
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
            },
            Some(winit::VirtualKeyCode::S) | Some(winit::VirtualKeyCode::Down) => {
                // Backward
                println!("Input! backward");
                self.position = self.position.sub(self.front * velocity);
            },
            Some(winit::VirtualKeyCode::A) | Some(winit::VirtualKeyCode::Left) => {
                // Left
                println!("Input! left");
                self.position = self.position.sub(self.right * velocity);
            },
            Some(winit::VirtualKeyCode::D) | Some(winit::VirtualKeyCode::Right) => {
                // Right
                println!("Input! right");
                self.position = self.position.add(self.right * velocity);
            },
            _ => (),
        };
    }
}
