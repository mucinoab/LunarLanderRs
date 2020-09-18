use std::f32::consts::PI;
use std::path;

const THRUST: f32 = 15.0;
const MAX_PHYSICS_VEL: f32 = 200.0;
const GRAVITY: f32 = 0.1;
const PLAYER_TURN_RATE: f32 = 1.5;

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics,
    graphics::{screenshot, DrawParam},
    nalgebra as na, timer, Context, GameResult,
};

use ncollide2d::{na as nac, shape};
use rand::prelude::*;
use simdnoise::NoiseBuilder;

type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

struct MainState {
    meshes: Vec<graphics::Mesh>,
    font: graphics::Font,
    ship: Ship,
    input: InputState,
    _mountain: shape::Polyline<f32>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let num_stars = 30;

        let (mesh_mountain, mountain) = build_mountain(ctx);
        let meshes = vec![build_stars(ctx, num_stars), mesh_mountain];

        let s = MainState {
            meshes,
            font: graphics::Font::default(),
            ship: Ship::new(ctx),
            input: InputState::default(),
            _mountain: mountain,
        };

        Ok(s)
    }
}

struct Ship {
    pos: Point2,
    facing: f32,
    velocity: Vector2,
    _bbox_size: f32,
    sprites: [graphics::Mesh; 4],
    thrust: bool,
    _armor: shape::Polyline<f32>,
}

impl Ship {
    fn new(ctx: &mut Context) -> Ship {
        let point = Point2::new(100.0, 100.0);

        Ship {
            pos: point,
            facing: PI,
            velocity: Vector2::new(55.0, 20.0),
            _bbox_size: 12.0,
            _armor: shape::Polyline::new(
                vec![
                    //TODO new shape
                    ncollide2d::na::Point2::new(0.0, 10.0),
                ],
                None,
            ),
            sprites: [
                graphics::Mesh::new_polyline(
                    ctx, //red big flame
                    graphics::DrawMode::stroke(0.5),
                    &[
                        Point2::new(0.0, -0.0) * 2.0,
                        Point2::new(-2.0, -1.0) * 2.0,
                        Point2::new(0.0, -6.0) * 2.0,
                        Point2::new(2.0, -1.0) * 2.0,
                        Point2::new(0.0, -0.0) * 2.0,
                    ],
                    graphics::Color::from_rgb(222, 3, 64),
                )
                .unwrap(),
                graphics::Mesh::new_polyline(
                    ctx, //orange middle flame
                    graphics::DrawMode::fill(),
                    &[
                        Point2::new(0.0, -0.0) * 2.0,
                        Point2::new(-2.0, -1.0) * 2.0,
                        Point2::new(0.0, -5.5) * 2.0,
                        Point2::new(2.0, -1.0) * 2.0,
                        Point2::new(0.0, -0.0) * 2.0,
                    ],
                    graphics::Color::from_rgb(255, 165, 0),
                )
                .unwrap(),
                graphics::Mesh::new_polyline(
                    ctx, // blue little flame
                    graphics::DrawMode::fill(),
                    &[
                        Point2::new(0.0, -0.0) * 2.0,
                        Point2::new(-0.5, -0.25) * 2.0,
                        Point2::new(0.0, -3.5) * 2.0,
                        Point2::new(0.5, -0.25) * 2.0,
                        Point2::new(0.0, -0.0) * 2.0,
                    ],
                    graphics::Color::from_rgb(26, 26, 255),
                )
                .unwrap(),
                graphics::Mesh::new_polyline(
                    ctx, //ship
                    graphics::DrawMode::stroke(1.0),
                    &[
                        Point2::new(18.0, -5.0),
                        Point2::new(22.0, -5.0),
                        Point2::new(20.0, -5.0),
                        Point2::new(15.0, 8.2),
                        Point2::new(10.0, 10.0),
                        Point2::new(10.0, 0.0),
                        Point2::new(15.0, 8.2),
                        Point2::new(10.0, 10.0),
                        Point2::new(5.0, 10.0),
                        Point2::new(7.0, 14.0),
                        Point2::new(7.0, 23.0),
                        Point2::new(-5.0, 23.0),
                        Point2::new(-5.0, 20.0),
                        Point2::new(-8.1, 17.1),
                        Point2::new(-8.1, 13.0),
                        Point2::new(-5.0, 13.0),
                        Point2::new(-5.0, 10.0),
                        Point2::new(-10.0, 10.0),
                        Point2::new(-15.0, 8.2),
                        Point2::new(-20.0, -5.0),
                        Point2::new(-22.0, -5.0),
                        Point2::new(-18.0, -5.0),
                        Point2::new(-20.0, -5.0),
                        Point2::new(-15.0, 8.2),
                        Point2::new(-10.0, 0.0),
                        Point2::new(-10.0, 10.0),
                        Point2::new(-10.0, 0.0),
                        Point2::new(10.0, 0.0),
                        Point2::new(10.0, 10.0),
                    ],
                    graphics::WHITE,
                )
                .unwrap(),
            ],

            thrust: false,
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let drawparams = graphics::DrawParam::new()
            .dest(self.pos)
            .rotation(-self.facing)
            .offset(Point2::new(0.5, 0.5));

        if self.thrust {
            for x in 0..3 {
                graphics::draw(ctx, &self.sprites[x], drawparams)?;
            }
        }

        graphics::draw(ctx, &self.sprites[3], drawparams)
    }
}

fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

fn player_thrust(actor: &mut Ship, dt: f32) {
    let direction_vector = vec_from_angle(actor.facing);
    let thrust_vector = direction_vector * THRUST;
    actor.velocity += thrust_vector * (dt);
}

fn update_actor_position(actor: &mut Ship, dt: f32) {
    let norm_sq = actor.velocity.norm_squared();
    if norm_sq > MAX_PHYSICS_VEL.powi(2) {
        actor.velocity = actor.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
    }
    let dv = actor.velocity * (dt);
    actor.pos += dv;
    actor.velocity.y += GRAVITY;
}

fn player_handle_input(actor: &mut Ship, input: &InputState, dt: f32) {
    actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

    if input.yaxis > 0.0 {
        player_thrust(actor, dt);
    }
}

struct InputState {
    xaxis: f32,
    yaxis: f32,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let (sx, sy) = graphics::size(ctx);
        const DESIRED_FPS: u32 = 60;
        let seconds = 1.0 / (DESIRED_FPS as f32);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            player_handle_input(&mut self.ship, &self.input, seconds);
            update_actor_position(&mut self.ship, seconds);
            bounds(&mut self.ship, sx, sy);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for m in &self.meshes {
            graphics::draw(ctx, m, DrawParam::default())?;
        }

        self.ship.draw(ctx)?;

        graphics::draw(
            ctx,
            &graphics::Text::new((
                format!(
                    "\nHORIZONTAL SPEED   {:.0}\
                     \nVERTICAL SPEED     {:.0}",
                    self.ship.velocity.x, self.ship.velocity.y,
                ),
                self.font,
                15.0,
            )),
            DrawParam::default(),
        )?;

        graphics::present(ctx).unwrap();
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Up | KeyCode::K => {
                self.input.yaxis = 1.0;
                self.ship.thrust = true;
            }

            KeyCode::Left | KeyCode::H => {
                self.input.xaxis = -1.0;
            }

            KeyCode::Right | KeyCode::L => {
                self.input.xaxis = 1.0;
            }

            KeyCode::Space => {
                self.ship.velocity.x = 0.0;
                self.ship.velocity.y = 0.0;
            }

            KeyCode::Escape => {
                let path = path::Path::new("/ss.png");
                screenshot(ctx)
                    .unwrap()
                    .encode(ctx, graphics::ImageFormat::Png, path)
                    .unwrap();
                event::quit(ctx)
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::Up | KeyCode::K => {
                self.input.yaxis = 0.0;
                self.ship.thrust = false;
            }

            KeyCode::Left | KeyCode::Right | KeyCode::L | KeyCode::H => {
                self.input.xaxis = 0.0;
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("", "")
        .window_mode(
            conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::True)
                .resizable(false)
                .maximized(true)
                .borderless(true)
                .dimensions(1366.0, 768.0),
        )
        .backend(conf::Backend::default().version(4, 6).gl());
    //.window_setup(conf::WindowSetup::default().samples(conf::NumSamples::from_u32(8).unwrap()));

    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}

fn build_stars(ctx: &mut Context, num_stars: usize) -> graphics::Mesh {
    let mb = &mut graphics::MeshBuilder::new();
    let mut rng = rand::thread_rng();

    let (max_x, mut max_y) = graphics::size(ctx);
    max_y = (max_y / 2.0) + 100.0;

    for _ in 0..num_stars {
        let x = rng.gen_range(0.0, max_x);
        let y = rng.gen_range(0.0, max_y);
        let z = rng.gen_range(1.0, 2.5);

        mb.line(
            &[Point2::new(x, y), Point2::new(x + 1.0, y + 1.0)],
            z,
            graphics::WHITE,
        )
        .unwrap();
    }

    mb.build(ctx).unwrap()
}

fn build_mountain(ctx: &mut Context) -> (graphics::Mesh, shape::Polyline<f32>) {
    let mb = &mut graphics::MeshBuilder::new();
    let (max_x, mut max_y) = graphics::size(ctx);

    let mut points_mesh: Vec<_> = Vec::with_capacity(max_x.ceil() as usize);
    let mut points_geometry: Vec<_> = Vec::with_capacity(max_x.ceil() as usize);

    let min_y = (max_y / 2.0) + 100.0;
    max_y = min_y + 150.0;

    let noise = NoiseBuilder::gradient_1d(max_x as _)
        .with_seed(rand::random::<i32>())
        .with_freq(0.01)
        .generate_scaled(min_y, max_y);

    for x in (0..max_x as usize).step_by(25) {
        points_mesh.push([x as f32, noise[x]]);
        points_geometry.push(ncollide2d::na::Point2::new(x as f32, noise[x]));
    }

    mb.polyline(
        graphics::DrawMode::stroke(1.0),
        &points_mesh,
        graphics::WHITE,
    )
    .unwrap();

    (
        mb.build(ctx).unwrap(),
        shape::Polyline::new(points_geometry, None),
    )
}

fn bounds(ship: &mut Ship, sx: f32, sy: f32) {
    if ship.pos.x > sx {
        ship.pos.x = 0.0;
    } else if ship.pos.x < 0.0 {
        ship.pos.x = sx;
    }

    if ship.pos.y < 0.0 {
        ship.pos.y = sy;
    } else if ship.pos.y > sy {
        ship.pos.y = 0.0;
    }
}

fn _collisions(ship: &Ship, mountain: &shape::Polyline<f32>) -> bool {
    let m = nac::Isometry2::new(nac::Vector2::new(100.0, 100.0), nac::zero());
    let s = nac::Isometry2::new(nac::Vector2::new(ship.pos.x, ship.pos.y), 1.0);

    let n = match ncollide2d::query::contact(&m, mountain, &s, &ship._armor, 1.0) {
        Some(n) => n.depth,
        _ => 0.0,
    };

    if n >= 0.0 {
        true
    } else {
        false
    }
}

//TODO
//Particulas?
//Colisiones, nave, linea
//numero de samples
