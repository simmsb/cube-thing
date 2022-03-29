use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use crate::animations::current_config;
use crate::backends::null::NullBackend;
use crate::render::DynDriver;

#[derive(Component)]
struct Coordinate {
    x: u8,
    y: u8,
    z: u8,
    mat: Handle<StandardMaterial>,
}

struct State {
    driver: DynDriver,
}

struct Lol<T>(Arc<RwLock<T>>);

impl<T: Inspectable + Send + Sync + 'static> Plugin for Lol<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Lol(self.0.clone()))
            .add_plugin(InspectorPlugin::<Lol<T>>::new_insert_manually());
    }
}

impl<T: Inspectable> Inspectable for Lol<T> {
    type Attributes = T::Attributes;

    fn ui_raw(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes) {
        self.0.write().unwrap().ui_raw(ui, options);
    }

    fn setup(app: &mut App) {
        T::setup(app);
    }

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        self.0.write().unwrap().ui(ui, options, context)
    }
}

fn setup(
    state: Res<State>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let light_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));

    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.8, 0.9, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    for (x, y, z, _pix) in state.driver.frame().pixels() {
        commands.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(
                (x as f32 - 4.0) * 4.0,
                (y as f32 - 4.0) * 4.0,
                (z as f32 - 4.0) * 4.0,
            ),
            ..Default::default()
        });

        let light_mat = materials.add(StandardMaterial {
            base_color: Color::ANTIQUE_WHITE,
            emissive: Color::ANTIQUE_WHITE,
            alpha_mode: AlphaMode::Blend,
            metallic: 0.5,
            ..Default::default()
        });

        commands
            .spawn()
            .insert(Coordinate {
                x,
                y,
                z,
                mat: light_mat.clone(),
            })
            .insert_bundle(PbrBundle {
                mesh: light_mesh.clone(),
                material: light_mat,
                transform: Transform::from_xyz(
                    (x as f32 - 4.0) * 4.0,
                    (y as f32 - 4.0) * 4.0,
                    (z as f32 - 4.0) * 4.0,
                ),
                ..Default::default()
            });
    }
}

fn update_driver_system(
    mut state: ResMut<State>,
    lights: Query<&Coordinate>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    state.driver.step();

    for coord in lights.iter() {
        let value = state
            .driver
            .frame()
            .get(coord.x as usize, coord.y as usize, coord.z as usize) as f32
            / 256.0;
        let mat = materials.get_mut(&coord.mat).unwrap();
        mat.base_color = Color::rgba(value, value, value, value * 0.5 + 0.5);
        mat.emissive = Color::rgba(value, value, value, value * 0.5 + 0.5);
    }
}

pub fn main() {
    let animation = Arc::new(RwLock::new(current_config()));

    let driver = DynDriver::new(animation.clone(), NullBackend);

    App::new()
        .insert_resource(State { driver })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_flycam::PlayerPlugin)
        .add_plugin(Lol(animation.clone()))
        .add_startup_system(setup)
        .add_system(update_driver_system)
        .run();
}
