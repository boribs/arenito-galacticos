pub mod arenito;
pub mod cans;
pub mod collision;
pub mod scenes;
pub mod sensor;
pub mod static_shape;

use arenito::{ArenitoConfig, ArenitoPlugin};
use bevy::{
    prelude::*,
    window::{ExitCondition, WindowResolution},
    winit::WinitSettings,
};
use clap::Parser;
use memmap;
use scenes::{SceneData, SceneLoaderPlugin};
use sensor::AISimMem;
use std::{fs::OpenOptions, io::Write};

/// CLI arguments
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// If set, main window's default size is small
    #[arg(short, long, default_value_t = false)]
    small_window: bool,
    /// Make Arenito's cameras windows visible
    #[arg(short = 'v', long, default_value_t = false)]
    visible_cameras: bool,
}

const SMALL_WINDOW_SIZE_WIDTH: f32 = 600.0;
const SMALL_WINDOW_SIZE_HEIGHT: f32 = 360.0;

fn main() {
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(AISimMem::MMAP_FILENAME)
    {
        Ok(f) => f,
        Err(_) => AISimMem::create_shareable_file(),
    };
    // Clear first bytes
    let _ = file.write(&[0, 0, 0]);

    let args = Args::parse();
    let window_res = if args.small_window {
        WindowResolution::new(SMALL_WINDOW_SIZE_WIDTH, SMALL_WINDOW_SIZE_HEIGHT)
    } else {
        WindowResolution::default()
    };

    let mut mmap = unsafe {
        memmap::MmapOptions::new()
            .map_mut(&file)
            .expect("Could not access data from file.")
    };

    App::new()
        .add_plugins(DefaultPlugins.build().set(WindowPlugin {
            exit_condition: ExitCondition::OnPrimaryClosed,
            primary_window: Some(Window {
                resolution: window_res,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(WinitSettings {
            return_from_run: true,
            ..default()
        })
        .insert_resource(AISimMem::new(&mut mmap))
        .add_plugins((
            SceneLoaderPlugin {
                scene_data: SceneData::default(),
                draw_can_collision_sphere: true,
                draw_obstacle_collision_mesh: true,
            },
            ArenitoPlugin {
                enable_can_eating: true,
                arenito_config: ArenitoConfig {
                    visible_cameras: args.visible_cameras,
                    ..default()
                },
            },
        ))
        .run();
}
