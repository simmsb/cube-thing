use bevy::prelude::*;
use std::collections::HashMap;

use crate::animations::current_config;
use crate::render::Driver;

struct State {
    driver: Driver,
    materials: HashMap<(u8, u8, u8), Handle<StandardMaterial>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cl: ResMut<bevy_config_cam::CamLogic>,
) {
    let animation = current_config();

    let driver = Driver::new(animation);

    let mut materials_store = HashMap::new();

    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    for (x, y, z, _pix) in driver.frame().pixels() {
        let material = materials.add(StandardMaterial {
            metallic: 0.5,
            ..Default::default()
        });

        materials_store.insert((x, y, z), material.clone());

        commands.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material,
            transform: Transform::from_xyz(x as f32 * 4.0, y as f32 * 4.0, z as f32 * 4.0),
            ..Default::default()
        });
    }

    commands.insert_resource(State {
        driver,
        materials: materials_store,
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 100.0),
        ..Default::default()
    });

    let target = commands
        .spawn()
        .insert(Transform::from_xyz(16.0, 16.0, 16.0))
        .id();

    cl.target = Some(target);
}

fn update_driver_system(mut state: ResMut<State>, mut materials: ResMut<Assets<StandardMaterial>>) {
    state.driver.step();

    for (x, y, z, pix) in state.driver.frame().pixels() {
        let c = pix as f32 / 256.0;

        let handle = state.materials.get(&(x, y, z)).unwrap();
        let mat = materials.get_mut(handle).unwrap();

        mat.base_color = Color::rgb(c, c, c);
        mat.emissive = Color::rgb(c, c, c);
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_config_cam::ConfigCam)
        .add_startup_system(setup)
        .add_system(update_driver_system)
        .run();
}
