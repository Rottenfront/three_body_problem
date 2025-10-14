use std::time::Instant;

use egui_macroquad::egui::{self, TextEdit};
use macroquad::prelude::*;

// use glam::vec3;

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;

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

    fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position = vec3(x, y, z);
    }

    fn accelerate(&mut self, a: Vec3, dt: f32) {
        self.velocity += a * dt;
    }

    fn move_sphere(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

struct System {
    bodies: Vec<Body>,
}

fn accelerate(body1: &mut Body, body2: &Body, dt: f32) {
    fn find_acceleration(body1: &Body, body2: &Body) -> Vec3 {
        const G: f32 = 6.67430e-8;
        const EPS: f32 = 1e-9;
        let vector = body2.position - body1.position;
        let r = vector.length();

        let normalized = vector.normalize();
        let acceleration = if r < EPS {
            0.0
        } else {
            G * body2.mass / (r * r)
        };
        normalized * acceleration
    }
    body1.accelerate(find_acceleration(body1, body2), dt);
}

impl System {
    fn accelerate(&mut self, dt: f32) {
        for i in 0..self.bodies.len() {
            let mut current = self.bodies[i].clone();
            for j in 0..self.bodies.len() {
                if i != j {
                    accelerate(&mut current, &self.bodies[j], dt);
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
}

#[macroquad::main(conf)]
async fn main() {
    let mut x = 0.0;
    let mut switch = false;
    let bounds = 8.0;

    let world_up = vec3(0.0, 1.0, 0.0);
    let mut yaw: f32 = 1.18;
    let mut pitch: f32 = 0.0;

    let mut front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let mut right = front.cross(world_up).normalize();
    let mut up = right.cross(front).normalize();

    let mut position = vec3(0.0, 1.0, 0.0);
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    let mut system = System { bodies: vec![] };
    let mut running = false;
    let mut prev_instant = Instant::now();

    let mut selected_body = None;

    loop {
        let delta = get_frame_time();

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_down(KeyCode::Up) {
            position += front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Down) {
            position -= front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            position -= right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Right) {
            position += right * MOVE_SPEED;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;

        last_mouse_position = mouse_position;

        if grabbed {
            yaw += mouse_delta.x * delta * LOOK_SPEED;
            pitch += mouse_delta.y * delta * -LOOK_SPEED;

            pitch = if pitch > 1.5 { 1.5 } else { pitch };
            pitch = if pitch < -1.5 { -1.5 } else { pitch };

            front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            right = front.cross(world_up).normalize();
            up = right.cross(front).normalize();

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }
        }

        clear_background(BLACK);

        // Going 3d!

        set_camera(&Camera3D {
            position: position,
            up: up,
            target: position + front,
            ..Default::default()
        });

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
                body.position / 1000.0,
                body.radius / 200.0,
                None,
                Color::new(body.color[0], body.color[1], body.color[2], 1.0),
            );
        }

        set_default_camera();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Simulation Controls")
                .default_pos((10.0, 10.0))
                .show(egui_ctx, |ui| {
                    ui.heading("Simulation");
                    if ui.checkbox(&mut running, "Run simulation").clicked() {
                        prev_instant = Instant::now();
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
            selected_body = None;
        }

        next_frame().await
    }
}
