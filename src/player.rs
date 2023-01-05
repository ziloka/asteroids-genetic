use std::{f32::consts::PI, f64::consts::TAU, iter};

use macroquad::{prelude::*, rand::gen_range};
use nalgebra::{max, partial_max, partial_min};

use crate::{asteroids::Asteroid, nn::NN};
#[derive(Default)]
pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    acc: f32,
    pub dir: Vec2,
    rot: f32,
    drag: f32,
    bullets: Vec<Bullet>,
    asteroid: Option<Asteroid>,
    asteroid_data: Vec<(f32, f32, f32)>,
    last_shot: u8,
    shot_interval: u8,
    pub brain: Option<NN>,
    debug: bool,
    alive: bool,
    pub color: Option<Color>,
    pub lifespan: u32,
    pub shots: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            dir: vec2(0., -1.),
            rot: 1.5 * PI,

            // Change scaling when passing inputs if this is changed
            drag: 0.001,
            shot_interval: 18,
            alive: true,
            debug: false,
            shots: 4,

            ..Default::default()
        }
    }

    pub fn simulate(brain: Option<NN>) -> Self {
        let mut p = Player::new();
        if let Some(brain) = brain {
            // assert_eq!(
            //     brain.config[0] - 1,
            //     8 + 5,
            //     "NN input size must match max_asteroids"
            // );
            p.brain = Some(brain);
        } else {
            p.brain = Some(NN::new(vec![4, 8, 8, 4]));
        }
        p
    }

    pub fn check_player_collision(&mut self, asteroid: &Asteroid) -> bool {
        // let directions = [
        //     vec2(0., -screen_height()),
        //     vec2(0., screen_height()),
        //     vec2(-screen_width(), 0.),
        //     vec2(screen_width(), 0.),
        // ];
        // let mut nearest = asteroid.pos - self.pos;
        // for dir in directions {
        //     if (asteroid.pos - self.pos + dir).length_squared() < nearest.length_squared() {
        //         nearest = asteroid.pos - self.pos + dir;
        //     }
        // }
        self.asteroid_data.push((
            ((asteroid.pos - self.pos).length() - asteroid.radius) / screen_width(),
            self.dir.angle_between(asteroid.pos - self.pos),
            (asteroid.vel - self.vel).length(),
        ));
        // if self.asteroid.is_none()
        //     || (asteroid.pos).distance_squared(self.pos)
        //         < self
        //             .asteroid
        //             .as_ref()
        //             .unwrap()
        //             .pos
        //             .distance_squared(self.pos)
        // {
        //     self.asteroid = Some(asteroid.clone());
        // }
        // ]);
        // let v = asteroid.pos - self.pos;
        // for i in 0..4 {
        //     let dir = Vec2::from_angle(PI / 4. * i as f32).rotate(self.dir);
        //     let cross = v.perp_dot(dir);
        //     let dot = v.dot(dir);
        //     if cross.abs() <= asteroid.radius {
        //         self.raycasts[if dot >= 0. { i } else { i + 4 }] = *partial_max(
        //             &self.raycasts[if dot >= 0. { i } else { i + 4 }],
        //             &(1. / (dot.abs()
        //                 - (asteroid.radius * asteroid.radius - cross * cross).sqrt())),
        //         )
        //         .unwrap();
        //     }
        // }
        if asteroid.check_collision(self.pos, 8.) || self.lifespan > 4000 {
            self.alive = false;
            return true;
        }
        false
    }

    pub fn check_bullet_collisions(&mut self, asteroid: &mut Asteroid) -> bool {
        for bullet in &mut self.bullets {
            if asteroid.check_collision(bullet.pos, 0.) {
                asteroid.alive = false;
                bullet.alive = false;
                self.asteroid = None;
                return true;
            }
        }
        false
    }

    pub fn update(&mut self) {
        self.lifespan += 1;
        self.last_shot += 1;
        self.acc = 0.;
        let mut keys = vec![false, false, false, false];
        let mut inputs = vec![
            // (self.asteroid.as_ref().unwrap().pos - self.pos).length() / screen_width(),
            // self.dir
            //     .angle_between(self.asteroid.as_ref().unwrap().pos - self.pos),
            // self.vel.x / 11.,
            // self.vel.y / 11.,
            self.rot / TAU as f32,
            // self.rot.sin(),
            // self.rot.cos(),
        ];

        self.asteroid_data
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.asteroid_data.resize(1, (0., 0., 0.));
        inputs.append(
            &mut self
                .asteroid_data
                .iter()
                .map(|(d, a, h)| vec![*d, *a, *h])
                .flatten()
                .collect::<Vec<_>>(),
        );
        // println!("{:?}", inputs);
        // let inputs = self.raycasts.clone();
        // inputs.append(self.asteroids_data.as_mut());
        if let Some(brain) = &self.brain {
            // println!("{:?}", brain.feed_forward(inputs.clone()));

            keys = brain.feed_forward(inputs).iter().map(|&x| x > 0.).collect();
        }
        if is_key_down(KeyCode::Right) && self.debug || keys[0] {
            self.rot = (self.rot + 0.1 + TAU as f32) % TAU as f32;
            self.dir = vec2(self.rot.cos(), self.rot.sin());
        }
        if is_key_down(KeyCode::Left) && self.debug || keys[1] {
            self.rot = (self.rot - 0.1 + TAU as f32) % TAU as f32;
            self.dir = vec2(self.rot.cos(), self.rot.sin());
        }
        if is_key_down(KeyCode::Up) && self.debug || keys[2] {
            // Change scaling when passing inputs if this is changed
            self.acc = 0.14;
        }
        if is_key_down(KeyCode::Space) && self.debug || keys[3] {
            if self.last_shot > self.shot_interval {
                self.last_shot = 0;
                self.shots += 1;
                self.bullets.push(Bullet {
                    pos: self.pos + self.dir * 20.,
                    vel: self.dir * 8.5 + self.vel,
                    alive: true,
                });
            }
        }

        if is_key_pressed(KeyCode::D) {
            self.debug = !self.debug;
        }

        self.vel += self.acc * self.dir - self.drag * self.vel.length() * self.vel;
        self.pos += self.vel;
        if self.pos.x.abs() > screen_width() * 0.5 + 10. {
            self.pos.x *= -1.;
        }
        if self.pos.y.abs() > screen_height() * 0.5 + 10. {
            self.pos.y *= -1.;
        }

        for bullet in &mut self.bullets {
            bullet.update();
        }
        self.bullets.retain(|b| {
            b.alive && b.pos.x.abs() * 2. < screen_width() && b.pos.y.abs() * 2. < screen_height()
        });
        // self.draw();
        self.asteroid = None;
        self.asteroid_data.clear();
    }

    pub fn draw(&self) {
        let color = match self.color {
            Some(c) => c,
            None => Color::new(1., 1., 1., 0.3),
        };
        let p1 = self.pos + self.dir * 20.;
        let p2 = self.pos + self.dir.rotate(vec2(-18., -12.667));
        let p3 = self.pos + self.dir.rotate(vec2(-18., 12.667));
        let p4 = self.pos + self.dir.rotate(vec2(-10., -10.));
        let p5 = self.pos + self.dir.rotate(vec2(-10., 10.));
        let p6 = self.pos + self.dir * -25.;
        let p7 = self.pos + self.dir.rotate(vec2(-10., -6.));
        let p8 = self.pos + self.dir.rotate(vec2(-10., 6.));
        draw_line(p1.x, p1.y, p2.x, p2.y, 2., color);
        draw_line(p1.x, p1.y, p3.x, p3.y, 2., color);
        draw_line(p4.x, p4.y, p5.x, p5.y, 2., color);
        if self.acc > 0. && gen_range(0., 1.) < 0.4 {
            draw_triangle_lines(p6, p7, p8, 2., color);
        }
        if self.debug {
            // if self.asteroid.is_some() {
            //     draw_circle_lines(
            //         self.asteroid.as_ref().unwrap().pos.x,
            //         self.asteroid.as_ref().unwrap().pos.y,
            //         self.asteroid.as_ref().unwrap().radius,
            //         1.,
            //         GRAY,
            //     );
            // }
            let p = self.pos
                + self.dir.rotate(Vec2::from_angle(self.asteroid_data[0].1))
                    * self.asteroid_data[0].0
                    * screen_width();
            draw_line(
                self.pos.x, self.pos.y,
                // self.asteroid.as_ref().unwrap().pos.x,
                // self.asteroid.as_ref().unwrap().pos.y,
                p.x, p.y, 1., GRAY,
            );

            // for (i, r) in self.raycasts.iter().enumerate() {
            //     let dir = Vec2::from_angle(PI / 4. * i as f32).rotate(self.dir);
            //     draw_line(
            //         self.pos.x,
            //         self.pos.y,
            //         self.pos.x + dir.x / r,
            //         self.pos.y + dir.y / r,
            //         1.,
            //         GRAY,
            //     );
            // }
        }

        for bullet in &self.bullets {
            bullet.draw();
        }
    }
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    alive: bool,
}

impl Bullet {
    fn update(&mut self) {
        self.pos += self.vel;
    }
    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, 2., WHITE);
    }
}
