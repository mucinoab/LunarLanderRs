use std::io::{self, Write};

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, DrawMode};
use ggez::input::keyboard;
use ggez::{timer, Context, GameResult};

use ggez::nalgebra as na;
use rand::prelude::*;
type Point2 = na::Point2<f32>;
//use cgmath;

struct MainState {
    position_x: f32,
    stars: Vec<Line>,
}

impl MainState {
    fn new(ctx: &Context) -> GameResult<MainState> {
        let num_stars = 50;
        let mut stars: Vec<Line> = Vec::with_capacity(num_stars);
        let mut rng = rand::thread_rng();
        let (max_x, max_y) = graphics::size(ctx);

        for _ in 0..num_stars {
            let x = rng.gen_range(0.0, max_x);
            let y = rng.gen_range(0.0, max_y / 2.0);
            stars.push(Line {
                points: [x, y, x + 1.0, y + 1.0],
                width: 1.0,
                color: graphics::WHITE,
            })
        }

        let s = MainState {
            position_x: 0.0,
            stars,
        };

        Ok(s)
    }
}

struct Line {
    points: [f32; 4],
    width: f32,
    color: graphics::Color,
}

struct _Ship {
    px: f32,
    py: f32,
    speed_x: f32,
    speed_y: f32,
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Increase or decrease `position_x` by 0.5, or by 5.0 if Shift is held.
        //
        if timer::ticks(ctx) % 100 == 0 {
            print!("\rAverage FPS: {}", timer::fps(ctx));
            io::stdout().flush().unwrap();
        }
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

        for star in &self.stars {
            let li = graphics::Mesh::new_line(
                ctx,
                &[
                    [star.points[0], star.points[1]],
                    [star.points[2], star.points[3]],
                ],
                star.width,
                star.color,
            )?;
            graphics::draw(ctx, &li, graphics::DrawParam::default())?;
        }

        let rotation = timer::ticks(ctx) % 1000;

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::stroke(3.0),
            Point2::new(0.0, 0.0),
            100.0,
            5.0,
            graphics::WHITE,
        )?;

        graphics::draw(
            ctx,
            &circle,
            (Point2::new(400.0, 300.0), rotation as f32, graphics::WHITE),
        )?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        match key {
            // Quit if Shift+Ctrl+Q is pressed.
            KeyCode::Q => {
                if mods.contains(KeyMods::SHIFT & KeyMods::CTRL) {
                    println!("Terminating!");
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
    let state = &mut MainState::new(&ctx)?;
    event::run(ctx, event_loop, state)
}
