pub mod arenito;
pub mod cans;
pub mod collision;
pub mod scenes;
pub mod sensor;
pub mod static_shape;

use arenito::ArenitoPlugin;
use bevy::{prelude::*, window::ExitCondition, winit::WinitSettings};
use memmap;
use scenes::{SceneLoaderPlugin, SceneName};
use sensor::AISimMem;
use std::{fs::OpenOptions, io::Write};

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
                name: SceneName::Deposit,
                display_can_collision_sphere: true,
            },
            ArenitoPlugin {
                enable_can_eating: true,
            },
        ))
        .run();
}
