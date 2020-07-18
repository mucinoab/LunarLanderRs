use std::f32::consts::PI;

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics,
    graphics::DrawParam,
    nalgebra as na, timer, Context, GameResult,
};
use ncollide2d::{nalgebra as nac, shape};
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
    sprite: graphics::Mesh,
    armor: shape::Polyline<f32>,
}

impl Ship {
    fn new(ctx: &mut Context) -> Ship {
        let point = Point2::new(100.0, 100.0);

        Ship {
            pos: point,
            facing: PI,
            velocity: na::zero(),
            _bbox_size: 12.0,
            armor: shape::Polyline::new(
                vec![
                    ncollide2d::nalgebra::Point2::new(0.0, 10.0),
                    ncollide2d::nalgebra::Point2::new(-10.0, -10.0),
                    ncollide2d::nalgebra::Point2::new(0.0, -5.0),
                    ncollide2d::nalgebra::Point2::new(10.0, -10.0),
                    ncollide2d::nalgebra::Point2::new(0.0, 10.0),
                ],
                None,
            ),
            sprite: graphics::Mesh::new_polyline(
                ctx,
                graphics::DrawMode::stroke(2.0),
                &[
                    Point2::new(0.0, 10.0),
                    Point2::new(-10.0, -10.0),
                    Point2::new(0.0, -5.0),
                    Point2::new(10.0, -10.0),
                    Point2::new(0.0, 10.0),
                ],
                graphics::WHITE,
            )
            .unwrap(),
        }
    }
}

fn draw_ship(ship: &Ship, ctx: &mut Context) -> GameResult {
    let drawparams = graphics::DrawParam::new()
        .dest(ship.pos)
        .rotation(ship.facing)
        .offset(Point2::new(0.5, 0.5));
    graphics::draw(ctx, &ship.sprite, drawparams)
}

fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

fn player_thrust(actor: &mut Ship, dt: f32) {
    let thrust: f32 = 500.0;
    let direction_vector = vec_from_angle(actor.facing);
    let thrust_vector = direction_vector * thrust;
    actor.velocity += thrust_vector * (dt);
}

fn update_actor_position(actor: &mut Ship, dt: f32) {
    const MAX_PHYSICS_VEL: f32 = 500.0;
    let norm_sq = actor.velocity.norm_squared();
    if norm_sq > MAX_PHYSICS_VEL.powi(2) {
        actor.velocity = actor.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
    }
    let dv = actor.velocity * (dt);
    actor.pos += dv;
}

fn player_handle_input(actor: &mut Ship, input: &InputState, dt: f32) {
    const PLAYER_TURN_RATE: f32 = 1.0;
    actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

    if input.yaxis == 1.0 {
        player_thrust(actor, dt);
    } else if input.yaxis == -1.0 {
        player_thrust(actor, -dt);
    }
}

fn collisions(ship: &Ship, mountain: &shape::Polyline<f32>) -> bool {
    let m = nac::Isometry2::new(nac::Vector2::new(100.0, 100.0), nac::zero());
    let s = nac::Isometry2::new(nac::Vector2::new(ship.pos.x, ship.pos.y), 1.0);

    let n = match ncollide2d::query::contact(&m, mountain, &s, &ship.armor, 1.0) {
        Some(n) => n.depth,
        _ => 0.0,
    };

    if n >= 0.0 {
        true
    } else {
        false
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
        const DESIRED_FPS: u32 = 60;
        let (sx, sy) = graphics::size(ctx);
        let seconds = 1.0 / (DESIRED_FPS as f32);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            player_handle_input(&mut self.ship, &self.input, seconds);
            update_actor_position(&mut self.ship, seconds);

            if self.ship.pos.x > sx {
                self.ship.pos.x = 0.0;
            } else if self.ship.pos.x < 0.0 {
                self.ship.pos.x = sx;
            }

            if self.ship.pos.y < 0.0 {
                self.ship.pos.y = sy;
            } else if self.ship.pos.y > sy {
                self.ship.pos.y = 0.0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for m in &self.meshes {
            graphics::draw(ctx, m, DrawParam::default())?;
        }

        draw_ship(&self.ship, ctx)?;

        graphics::draw(
            ctx,
            &graphics::Text::new((
                format!(
                    "FPS{:.0}\nPos({:.0},{:.0})\nFacing{:.4}\nVel({:.2},{:.2}\nCol{})",
                    timer::fps(ctx),
                    self.ship.pos.x,
                    self.ship.pos.x,
                    self.ship.facing % (2.0 * PI),
                    self.ship.velocity.x,
                    self.ship.velocity.y,
                    collisions(&self.ship, &self._mountain),
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
            KeyCode::Up => {
                self.input.yaxis = 1.0;
            }

            KeyCode::Down => {
                self.input.yaxis = -1.0;
            }

            KeyCode::Left => {
                self.input.xaxis = 1.0;
            }

            KeyCode::Right => {
                self.input.xaxis = -1.0;
            }

            KeyCode::Space => {
                self.ship.velocity.x = 0.0;
                self.ship.velocity.y = 0.0;
            }

            KeyCode::Escape => event::quit(ctx),
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::Up | KeyCode::Down => {
                self.input.yaxis = 0.0;
            }

            KeyCode::Left | KeyCode::Right => {
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
                .resizable(true)
                .borderless(true)
                .dimensions(1366.0, 768.0),
        )
        .window_setup(conf::WindowSetup::default().samples(conf::NumSamples::from_u32(8).unwrap()))
        .backend(conf::Backend::default().version(4, 6));

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
        .generate_scaled(min_y, max_y);

    for x in (0..max_x as usize).step_by(15) {
        points_mesh.push([x as f32, noise[x]]);
        points_geometry.push(ncollide2d::nalgebra::Point2::new(x as f32, noise[x]));
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

//To_Do
//Particulas?
//Colisiones, nave, linea
//movimiento
//numero de samples
