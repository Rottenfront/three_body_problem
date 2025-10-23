use std::time::Instant;

use egui_macroquad::egui::{self, TextEdit};
use macroquad::prelude::*;

// use glam::vec3;

const MOVE_SPEED: f32 = 0.01;
const LOOK_SPEED: f32 = 0.1;
const POSITION_SCALE: f32 = 0.001;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Three-body problem"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[derive(Clone, Copy)]
struct Body {
    mass: f32,
    radius: f32,
    position: Vec3,
    velocity: Vec3,
    color: [f32; 3],
}

impl Body {
    fn new() -> Self {
        Self {
            mass: 1000000000.0,
            radius: 5.0,
            position: vec3(0.0, 0.0, 0.0),
            velocity: vec3(0.0, 0.0, 0.0),
            color: [1.0, 1.0, 1.0],
        }
    }

    fn accelerate(&mut self, a: Vec3, dt: f32) {
        self.velocity += a * dt;
    }

    fn find_acceleration(&self, other: &Self) -> Vec3 {
        const G: f32 = 6.67430e-8;
        const EPS: f32 = 1e-9;
        let vector = other.position - self.position;
        let r = vector.length();

        let normalized = vector.normalize();
        let acceleration = if r < EPS {
            0.0
        } else {
            G * other.mass / (r * r)
        };
        normalized * acceleration
    }

    fn accelerate_by_body(&mut self, other: &Self, dt: f32) {
        self.accelerate(self.find_acceleration(other), dt);
    }

    fn move_sphere(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

struct System {
    bodies: Vec<Body>,
}

impl System {
    fn accelerate(&mut self, dt: f32) {
        for i in 0..self.bodies.len() {
            let mut current = self.bodies[i].clone();
            for j in 0..self.bodies.len() {
                if i != j {
                    current.accelerate_by_body(&self.bodies[j], dt);
                }
            }
            self.bodies[i] = current;
        }
    }

    fn move_bodies(&mut self, dt: f32) {
        for body in &mut self.bodies {
            body.move_sphere(dt);
        }
    }

    fn mass_center(&self) -> Vec3 {
        if self.bodies.is_empty() {
            vec3(0.0, 0.0, 0.0)
        } else {
            self.bodies
                .iter()
                .fold(Vec3::ZERO, |acc, body| acc + body.position * body.mass)
                / self.bodies.iter().fold(0.0, |acc, body| acc + body.mass)
                * POSITION_SCALE
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusPoint {
    None,
    MassCenter,
    Body(usize),
}

struct Camera {
    x: f32,
    switch: bool,
    bounds: f32,
    world_up: Vec3,
    yaw: f32,
    pitch: f32,
    radius: f32,
    front: Vec3,
    right: Vec3,
    up: Vec3,

    position: Vec3,
    last_mouse_position: Vec2,

    grabbed: bool,
}

impl Camera {
    fn new() -> Self {
        let x = 0.0;
        let switch = false;
        let bounds = 8.0;
        let world_up = vec3(0.0, 1.0, 0.0);
        let yaw: f32 = 1.18;
        let pitch: f32 = 0.0;
        let radius = 5.0;

        let front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();
        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();

        let position = vec3(0.0, 0.0, 0.0);
        let last_mouse_position: Vec2 = mouse_position().into();

        let grabbed = true;
        set_cursor_grab(grabbed);
        show_mouse(false);
        Self {
            yaw,
            pitch,
            radius,
            front,
            right,
            up,
            position,
            last_mouse_position,
            grabbed,
            x,
            switch,
            bounds,
            world_up,
        }
    }

    fn update_free(&mut self) {
        if is_key_down(KeyCode::W) {
            self.position += self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            self.position -= self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            self.position -= self.right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            self.position += self.right * MOVE_SPEED;
        }

        if is_key_down(KeyCode::Q) {
            self.position += self.up * MOVE_SPEED;
        }
        if is_key_down(KeyCode::E) {
            self.position -= self.up * MOVE_SPEED;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - self.last_mouse_position;

        self.last_mouse_position = mouse_position;

        if self.grabbed {
            let delta = get_frame_time();
            self.yaw += mouse_delta.x * delta * LOOK_SPEED;
            self.pitch += mouse_delta.y * delta * -LOOK_SPEED;

            self.pitch = self.pitch.clamp(-1.5, 1.5);

            self.front = vec3(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            )
            .normalize();

            self.right = self.front.cross(self.world_up).normalize();
            self.up = self.right.cross(self.front).normalize();

            self.x += if self.switch { 0.04 } else { -0.04 };
            if self.x >= self.bounds || self.x <= -self.bounds {
                self.switch = !self.switch;
            }
        }
    }

    fn update_with_point(&mut self, focus_point: Vec3) {
        let delta = get_frame_time();
        if is_key_down(KeyCode::A) {
            self.yaw -= LOOK_SPEED * delta;
        }
        if is_key_down(KeyCode::D) {
            self.yaw += LOOK_SPEED * delta;
        }
        if is_key_down(KeyCode::Q) {
            self.pitch += LOOK_SPEED * delta;
        }
        if is_key_down(KeyCode::E) {
            self.pitch -= LOOK_SPEED * delta;
        }

        // zoom in/out
        if is_key_down(KeyCode::W) {
            self.radius -= MOVE_SPEED * 2.0;
        }
        if is_key_down(KeyCode::S) {
            self.radius += MOVE_SPEED * 2.0;
        }
        self.position = focus_point
            + vec3(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            ) * self.radius;
        self.front = (focus_point - self.position).normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    fn apply_self(&self) {
        set_camera(&Camera3D {
            position: self.position,
            up: self.up,
            target: self.position + self.front,
            ..Default::default()
        });
    }

    fn draw_camera_gizmo(&self) {
        let axes = [
            (vec3(1.0, 0.0, 0.0), RED, "X"),
            (vec3(0.0, 1.0, 0.0), GREEN, "Y"),
            (vec3(0.0, 0.0, 1.0), BLUE, "Z"),
        ];

        let view_rot = Mat3::from_cols(self.right, self.up, -self.front);

        let base = vec2(screen_width() - 80.0, 80.0);
        let scale = 40.0;

        for (axis, color, label) in axes.iter() {
            let dir = view_rot * *axis;
            let end = base + vec2(dir.x, -dir.y) * scale;
            draw_line(base.x, base.y, end.x, end.y, 2.0, *color);
            draw_text(label, end.x + 4.0, end.y + 4.0, 16.0, *color);
        }

        let pos_text = format!("X: {:.2}", self.position.x / POSITION_SCALE,);
        draw_text(&pos_text, base.x - 60.0, base.y + 60.0, 16.0, WHITE);

        let pos_text = format!("Y: {:.2}", self.position.y / POSITION_SCALE,);
        draw_text(&pos_text, base.x - 60.0, base.y + 76.0, 16.0, WHITE);
        let pos_text = format!("Z: {:.2}", self.position.z / POSITION_SCALE,);
        draw_text(&pos_text, base.x - 60.0, base.y + 92.0, 16.0, WHITE);
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut camera = Camera::new();

    let mut system = System { bodies: vec![] };
    let mut running = false;
    let mut prev_instant = Instant::now();

    let mut selected_body = None;

    let mut focused = FocusPoint::None;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            camera.grabbed = !camera.grabbed;
            set_cursor_grab(camera.grabbed);
            show_mouse(!camera.grabbed);
        }

        match focused {
            FocusPoint::None => camera.update_free(),
            FocusPoint::MassCenter => camera.update_with_point(system.mass_center()),
            FocusPoint::Body(body_index) => {
                if let Some(_) = selected_body {
                    focused = FocusPoint::None;
                    camera.update_free();
                } else {
                    camera.update_with_point(system.bodies[body_index].position);
                }
            }
        }
        selected_body = None;

        clear_background(BLACK);

        camera.apply_self();

        if running {
            let instant = Instant::now();
            let dt = (instant - prev_instant).as_secs_f32();

            system.accelerate(dt);
            system.move_bodies(dt);
        }

        // draw axes
        draw_line_3d(vec3(10000.0, 0.0, 0.0), vec3(-10000.0, 0.0, 0.0), RED);
        draw_line_3d(vec3(0.0, 10000.0, 0.0), vec3(0.0, -10000.0, 0.0), GREEN);
        draw_line_3d(vec3(0.0, 0.0, 10000.0), vec3(0.0, 0.0, -10000.0), BLUE);

        // draw bodies

        for body in &system.bodies {
            draw_sphere(
                body.position * POSITION_SCALE,
                body.radius / 200.0,
                None,
                Color::new(body.color[0], body.color[1], body.color[2], 1.0),
            );
        }

        set_default_camera();

        camera.draw_camera_gizmo();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Simulation Controls")
                .default_pos((10.0, 10.0))
                .show(egui_ctx, |ui| {
                    ui.heading("Simulation");
                    if ui.checkbox(&mut running, "Run simulation").clicked() {
                        prev_instant = Instant::now();
                    }
                    if ui.button("Free camera").clicked() {
                        focused = FocusPoint::None;
                    }
                    if ui.button("Focus camera on mass center").clicked() {
                        focused = FocusPoint::MassCenter;
                    }

                    ui.separator();

                    // --- Add body button ---
                    if ui.button("Add new body").clicked() {
                        system.bodies.push(Body::new());
                    }

                    ui.separator();

                    // --- List of existing bodies ---
                    for (i, body) in system.bodies.iter_mut().enumerate() {
                        ui.collapsing(&format!("Body #{}", i), |ui| {
                            macro_rules! number_line {
                                ($variable:expr, $text:expr) => {
                                    let mut buffer = format!("{}", $variable);

                                    if ui
                                        .add(TextEdit::singleline(&mut buffer).hint_text($text))
                                        .changed()
                                    {
                                        if let Ok(parsed) = buffer.parse::<f32>() {
                                            $variable = parsed;
                                        }
                                    }
                                };
                            }
                            number_line!(body.mass, "Mass");
                            ui.add(egui::Slider::new(&mut body.radius, 0.5..=10.0).text("radius"));

                            ui.label("Position:");
                            number_line!(body.position.x, "x");
                            number_line!(body.position.y, "y");
                            number_line!(body.position.z, "z");
                            ui.label("Velocity:");
                            number_line!(body.velocity.x, "vx");
                            number_line!(body.velocity.y, "vy");
                            number_line!(body.velocity.z, "vz");
                            ui.color_edit_button_rgb(&mut body.color);

                            if ui.button("Focus body").clicked() {
                                focused = FocusPoint::Body(i);
                            }

                            // Remove body button
                            if ui.button("Remove this body").clicked() {
                                selected_body = Some(i);
                            }
                        });
                    }
                });
        });

        egui_macroquad::draw();

        if let Some(index) = selected_body {
            system.bodies.remove(index);
        }

        next_frame().await
    }
}
