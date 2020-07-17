use ggez::{
    conf,
    event::{self, EventHandler, KeyCode},
    graphics,
    graphics::DrawParam,
    input::keyboard,
    timer, Context, GameResult,
};

use rand::prelude::*;
use simdnoise::NoiseBuilder;

use ggez::nalgebra;
type Point2 = nalgebra::Point2<f32>;
type Vector2 = nalgebra::Vector2<f32>;

struct MainState {
    meshes: Vec<graphics::Mesh>,
    font: graphics::Font,
    ship: Ship,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let num_stars = 30;

        let meshes = vec![build_stars(ctx, num_stars), build_mountain(ctx)];

        let s = MainState {
            meshes,
            font: graphics::Font::default(),
            ship: Ship::new(ctx),
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

fn build_mountain(ctx: &mut Context) -> graphics::Mesh {
    let mb = &mut graphics::MeshBuilder::new();
    let (max_x, mut max_y) = graphics::size(ctx);
    let mut points: Vec<[f32; 2]> = Vec::with_capacity(max_x.ceil() as usize);

    let min_y = max_y / 2.0;
    max_y = min_y + 150.0;
    let noise = NoiseBuilder::gradient_1d(max_x as _).generate_scaled(min_y, max_y);

    for x in (0..max_x as usize).step_by(15) {
        points.push([x as f32, noise[x]]);
    }

    mb.polyline(graphics::DrawMode::stroke(1.0), &points, graphics::WHITE)
        .unwrap();

    mb.build(ctx).unwrap()
}

struct _Gui {
    score: i32,
    time: f32,
    altitude: f32,
    fuel: i32,
    horizontal_speed: f32,
    vertical_speed: f32,
}

struct Ship {
    pos: Point2,
    facing: f32,
    velocity: Vector2,
    ang_vel: f32,
    bbox_size: f32,
    sprite: graphics::Mesh,
}

impl Ship {
    fn new(ctx: &mut Context) -> Ship {
        Ship {
            pos: Point2::new(100.0, 100.0),
            facing: 0.0,
            velocity: nalgebra::zero(),
            ang_vel: 0.0,
            bbox_size: 12.0,
            sprite: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(Point2::origin()[0], Point2::origin()[1], 10.0, 10.0),
                graphics::WHITE,
            )
            .unwrap(),
        }
    }
}

fn draw_ship(ship: &Ship, ctx: &mut Context) -> GameResult {
    let drawparams = graphics::DrawParam::new()
        .dest(Point2::new(ship.pos[0], ship.pos[1]))
        .rotation(ship.facing);
    graphics::draw(ctx, &ship.sprite, drawparams)
}

fn wrap_actor_position(actor: &mut Ship, ctx: &Context) {
    let (sx, sy) = graphics::size(ctx);
    // Wrap screen
    let screen_x_bounds = sx / 2.0;
    let screen_y_bounds = sy / 2.0;
    if actor.pos.x > screen_x_bounds {
        actor.pos.x -= sx;
    } else if actor.pos.x < -screen_x_bounds {
        actor.pos.x += sx;
    };
    if actor.pos.y > screen_y_bounds {
        actor.pos.y -= sy;
    } else if actor.pos.y < -screen_y_bounds {
        actor.pos.y += sy;
    }
}

fn _world_to_screen_coords(screen_width: f32, screen_height: f32, point: Point2) -> Point2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point2::new(x, y)
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.ship.pos[0] += 3.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.ship.pos[0] -= 3.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.ship.pos[1] -= 3.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.ship.pos[1] += 3.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Space) {
            self.ship.facing -= 0.1;
        }

        wrap_actor_position(&mut self.ship, ctx);
        self.ship.pos[0] += 0.2;
        self.ship.pos[1] += 0.1;

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
            &graphics::Text::new((format!("{:.0}", timer::fps(ctx)), self.font, 35.0)),
            DrawParam::default(),
        )?;

        graphics::present(ctx).unwrap();
        timer::yield_now();
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("", "")
        .window_mode(
            conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::True)
                .resizable(true)
                .borderless(true),
        )
        .window_setup(conf::WindowSetup::default().samples(conf::NumSamples::from_u32(8).unwrap()))
        .backend(conf::Backend::default().version(4, 6));

    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
