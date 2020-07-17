use std::io::{self, Write};

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::input::keyboard;
use ggez::{graphics, timer, Context, GameResult};

use ggez::graphics::DrawParam;
use rand::prelude::*;
use simdnoise::*;

use ggez::nalgebra as na;
type Point2 = na::Point2<f32>;
//use cgmath;

struct MainState {
    position_x: f32,
    meshes: Vec<graphics::Mesh>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let num_stars = 35;

        let meshes = vec![build_stars(ctx, num_stars), build_mountain(ctx)];

        let s = MainState {
            position_x: 0.0,
            meshes,
        };

        Ok(s)
    }
}

fn build_stars(ctx: &mut Context, num_stars: usize) -> graphics::Mesh {
    let mb = &mut graphics::MeshBuilder::new();
    let mut rng = rand::thread_rng();

    let (max_x, mut max_y) = graphics::size(ctx);
    max_y /= 2.0;

    for _ in 0..num_stars {
        let x = rng.gen_range(0.0, max_x);
        let y = rng.gen_range(0.0, max_y);
        let z = rng.gen_range(1.0, 2.0);

        mb.line(
            &[Point2::new(x, y), Point2::new(x + 0.5, y + 0.5)],
            z,
            graphics::WHITE,
        )
        .unwrap();
    }

    mb.build(ctx).unwrap()
}

fn build_mountain(ctx: &mut Context) -> graphics::Mesh {
    let mb = &mut graphics::MeshBuilder::new();
    let (max_x, mut max_y) = graphics::size(ctx);
    let mut points: Vec<[f32; 2]> = Vec::with_capacity(max_x.ceil() as usize);

    let min_y = max_y / 2.0;
    max_y = min_y + 100.0;

    let noise = NoiseBuilder::gradient_1d(max_x as _).generate_scaled(min_y, max_y);

    for x in (0..max_x as usize).step_by(10) {
        points.push([x as f32, noise[x]]);
    }

    mb.polyline(graphics::DrawMode::stroke(1.0), &points, graphics::WHITE)
        .unwrap();

    mb.build(ctx).unwrap()
}

struct _Ship {
    px: f32,
    py: f32,
    speed_x: f32,
    speed_y: f32,
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::ticks(ctx) % 100 == 0 {
            print!("\rAverage FPS: {:.0}", timer::fps(ctx));
            io::stdout().flush().unwrap();
        }

        // Increase or decrease `position_x` by 0.5, or by 5.0 if Shift is held.
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
                self.position_x += 4.5;
            }
            self.position_x += 0.5;
        } else if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
                self.position_x -= 4.5;
            }
            self.position_x -= 0.5;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for m in &self.meshes {
            graphics::draw(ctx, m, DrawParam::default())?;
        }

        graphics::present(ctx).unwrap();
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        match key {
            // Quit if Shift+Ctrl+Q is pressed.
            KeyCode::Q => {
                if mods.contains(KeyMods::SHIFT & KeyMods::CTRL) {
                    println!("\nTerminating!");
                    event::quit(ctx);
                } else if mods.contains(KeyMods::SHIFT) || mods.contains(KeyMods::CTRL) {
                    println!("You need to hold both Shift and Control to quit.");
                } else {
                    println!("Now you're not even trying!");
                }
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
