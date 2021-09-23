use std::fmt;
use std::env;
use std::{thread, time};
use rand::Rng;
use std::collections::VecDeque;


use bevy::{
    math::Quat,
    prelude::*,
    ecs::component::Component,
};


const DEFAULT_WIDTH: &str = "10";
const DEFAULT_HEIGHT: &str = "10";
const DEFAULT_PERIOD: &str = "10";


struct MainTimer(Timer);

struct Board {
    cells: Box<Vec<u8>>,
    cells_cache: Box<Vec<u8>>,
    width: usize,
    height: usize,
    total_collapsed: i32,
    last_collapses: VecDeque<i32>,
    top_collapses: VecDeque<i32>,
    rng: rand::rngs::ThreadRng,
}

unsafe impl Send for Board {}
unsafe impl Sync for Board {}

impl Board {
    fn new(width: usize, height: usize) -> Board {
        Board {
            cells: Box::new(vec![0; width*height]),
            cells_cache: Box::new(vec![0; width*height]),
            width: width,
            height: height,
            total_collapsed: 0,
            last_collapses: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into_iter().collect(),
            top_collapses: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into_iter().collect(),
            rng: rand::thread_rng(),
        }
    }

    fn collapse(&mut self) -> i32 {
        let mut collapsed = 0;
    
        self.cells_cache.copy_from_slice(&self.cells);
        for y in 0..self.height {
            for x in 0..self.width {
                let index = x + (y*self.width);
                if self.cells_cache[index] >= 4 {
                    collapsed += 1;
                    self.cells[index] -= 4;
                    if x+1 < self.width {
                        self.cells[(x + 1) + (y * self.width)] += 1;
                    }
                    if x >= 1 {
                        self.cells[(x - 1) + (y * self.width)] += 1;
                    }
                    if y+1 < self.height {
                        self.cells[x + ((y + 1) * self.width)] += 1;
                    }
                    if y >= 1 {
                        self.cells[x + ((y - 1) * self.width)] += 1;
                    }
                }
            }
        }
        collapsed
    }

    fn populate(&mut self) {
        let len = self.cells.len() as i32;
        self.cells[self.rng.gen_range(0..len) as usize] += 1;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                output.push_str(match self.cells[x + (y*self.width)] {
                    0 => "0",  //" ",
                    1 => "1",  //"⸱",
                    2 => "2",  //"⁚",
                    3 => "3",  //"⸫",
                    4 => "4",  //"⸬",
                    _ => "+",  //"⸭"
                })
            }
            output.push_str("\n");
        }
        write!(f, "{}", output)
    }
}


fn tick(time: Res<Time>, mut timer: ResMut<MainTimer>, mut board: ResMut<Board>, mut sprites: Query<(&mut Sprite, &mut Transform)>) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut collapsed: i32 = board.collapse();
        if collapsed == 0 {
            if board.total_collapsed > 0 {
                board.last_collapses.pop_back();
                let total_collapsed = board.total_collapsed;
                board.last_collapses.push_front(total_collapsed);
                board.top_collapses.push_front(total_collapsed);
                board.top_collapses.make_contiguous().sort();
                board.top_collapses.make_contiguous().reverse();
                board.top_collapses.pop_back();
                board.total_collapsed = 0;
            }
            board.populate();
        } else {
            board.total_collapsed += collapsed;
        }
        // println!("\x1B[2J\x1B[1;1H{}\nlast : {:?}\ntop : {:?}", *board, board.last_collapses, board.top_collapses);
        println!("\x1B[2J\x1B[1;1Hlast : {:?}\ntop : {:?}\n[f] faster, [s] slower, period : {:?}", board.last_collapses, board.top_collapses, timer.0.duration());
        for (index, (_, mut sprite)) in sprites.iter_mut().enumerate() {
            sprite.scale = match board.cells[index] {
                0 => Vec3::splat(0.0),  //" ",
                1 => Vec3::splat(0.3333),  //"⸱",
                2 => Vec3::splat(0.6666),  //"⁚",
                3 => Vec3::splat(0.9999),  //"⸫",
                4 => Vec3::splat(1.3333),  //"⸬",
                _ => Vec3::splat(1.6666),  //"⸭"
            }
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let width = args.get(1).unwrap_or(&String::from(DEFAULT_WIDTH)).parse::<usize>().unwrap();
    let height = args.get(2).unwrap_or(&String::from(DEFAULT_HEIGHT)).parse::<usize>().unwrap();
    let sleep = time::Duration::from_millis(args.get(3).unwrap_or(&String::from(DEFAULT_PERIOD)).parse::<u64>().unwrap());


    let mut board = Board::new(width, height);


    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .insert_resource(MainTimer(Timer::new(sleep, true)))
        .insert_resource(board)
        .add_system(tick.system())
        .add_system(Keyboard_handling.system())
        .run()
}

fn Keyboard_handling(time: Res<Time>, mut timer: ResMut<MainTimer>, keys: Res<Input<KeyCode>>, btns: Res<Input<MouseButton>>) {
    unsafe {
        static mut last_press: time::Duration = time::Duration::from_millis(0);
        let now = time.time_since_startup();
    
        if keys.just_pressed(KeyCode::F) {
            let duration = timer.0.duration();
            let increment = if now - last_press > time::Duration::from_millis(500) {10} else {100};
            
            timer.0.set_duration(if duration < time::Duration::from_millis(increment) {
                time::Duration::from_millis(0)
            } else {
                duration - time::Duration::from_millis(increment)
            });
            last_press = now;
        }
        if keys.just_pressed(KeyCode::S) {
            let duration = timer.0.duration();
            let increment = if now - last_press > time::Duration::from_millis(500) {10} else {100};
        
            timer.0.set_duration(if duration > time::Duration::from_millis(10000) {
                time::Duration::from_millis(10000)
            } else {
                duration + time::Duration::from_millis(increment)
            });
            last_press = now;
        }
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let args: Vec<String> = env::args().collect();
    let width = args.get(1).unwrap_or(&String::from(DEFAULT_WIDTH)).parse::<f32>().unwrap();
    let height = args.get(2).unwrap_or(&String::from(DEFAULT_HEIGHT)).parse::<f32>().unwrap();

    let tile_size = Vec2::splat(16.0);
    let map_size = Vec2::new(width, height);

    let half_x = (map_size.x / 2.0) as i32;
    let half_y = (map_size.y / 2.0) as i32;

    let sprite_handle = materials.add(assets.load("pngwing.com.png").into());

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Transform::from_translation(Vec3::new(
            0.0, 0.0, 1000.0,
        )));

    for y in -half_y..half_y {
        for x in -half_x..half_x {
            let position = Vec2::new(x as f32, y as f32);
            let translation = (position * tile_size).extend(1.0);
            let rotation = Quat::from_rotation_z(0.0);
            let scale = Vec3::splat(1.0);

            commands.spawn().insert_bundle(SpriteBundle {
                material: sprite_handle.clone(),
                transform: Transform {
                    translation,
                    rotation,
                    scale,
                },
                sprite: Sprite::new(tile_size),
                ..Default::default()
            });
        }
    }
}
