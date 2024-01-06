pub mod arenito;
pub mod scenes;
pub mod sensor;
pub mod static_shape;

use arenito::ArenitoPlugin;
use bevy::{prelude::*, window::ExitCondition, winit::WinitSettings};
use scenes::{SceneLoaderPlugin, SceneName};
use sensor::AISimMem;
use shared_memory::*;

fn main() {
    let flink = "shmem_arenito";
    let shmem: Shmem = match ShmemConf::new()
        .size(AISimMem::MIN_REQUIRED_MEMORY)
        .flink(flink)
        .create()
    {
        Ok(m) => {
            println!("created successfully");
            m
        }
        Err(ShmemError::LinkExists) => {
            println!("already exists. connecting.");
            ShmemConf::new().size(100).flink(flink).open().unwrap()
        }
        Err(_) => panic!("you did something very wrong."),
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
        .insert_resource(AISimMem::new(&shmem))
        .add_plugins((
            SceneLoaderPlugin {
                name: SceneName::Basic,
            },
            ArenitoPlugin {
                img_width: 512.0,
                img_height: 512.0,
            },
        ))
        .run();
}
