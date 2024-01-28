pub mod arenito;
pub mod cans;
pub mod collision;
pub mod scenes;
pub mod sensor;
pub mod static_shape;

use std::fs::OpenOptions;
use arenito::ArenitoPlugin;
use bevy::{prelude::*, window::ExitCondition, winit::WinitSettings};
use scenes::{SceneLoaderPlugin, SceneName};
use sensor::AISimMem;
use memmap;

fn main() {
    // this could all be done in AISimMem::new()
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("file")
        .expect("Unable to open file.");

    let mut mmap = unsafe {
        memmap::MmapOptions::new()
            .map_mut(&file)
            .expect("Could not access data from file.")
    };

    App::new()
        .add_plugins(DefaultPlugins.build().set(WindowPlugin {
            exit_condition: ExitCondition::OnPrimaryClosed,
            ..default()
        }))
        .insert_resource(WinitSettings {
            return_from_run: true,
            ..default()
        })
        .insert_resource(AISimMem::new(&mut mmap))
        .add_plugins((
            SceneLoaderPlugin {
                name: SceneName::BasicCans,
                display_can_collision_sphere: true,
            },
            ArenitoPlugin {
                enable_can_eating: true,
            },
        ))
        .run();
}
