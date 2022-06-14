use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use palette::{Blend, LinSrgba, Srgba};

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

struct Lol<T, const NAME: &'static str>(Arc<RwLock<T>>);

impl<T: Inspectable + Send + Sync + 'static, const NAME: &'static str> Plugin for Lol<T, NAME> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Lol::<T, NAME>(self.0.clone()))
            .add_plugin(InspectorPlugin::<Lol<T, NAME>>::new_insert_manually());

        app.add_startup_system(|mut windows: ResMut<InspectorWindows>| {
            windows.window_data_mut::<Lol<T, NAME>>().name = NAME.to_owned();
        });
    }
}

impl<T: Inspectable, const NAME: &'static str> Inspectable for Lol<T, NAME> {
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
    let mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.6,
        subdivisions: 2,
    }));

    for (x, y, z, _pix) in state.driver.frame().pixels() {
        let material = materials.add(StandardMaterial {
            base_color: Color::rgba(0.0, 0.8, 0.9, 0.3),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        commands
            .spawn()
            .insert(Coordinate {
                x,
                y,
                z,
                mat: material.clone(),
            })
            .insert_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
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
            .get(coord.x as usize, coord.y as usize, coord.z as usize);
        let value = LinSrgba::new(0.3, 0.3, 0.3, 0.4).overlay(value);
        let mat = materials.get_mut(&coord.mat).unwrap();
        let colour = Srgba::from_linear(value);
        mat.base_color = Color::rgba(
            colour.red,
            colour.green,
            colour.blue,
            colour.alpha,
            // (colour.alpha * 0.5) + 0.5,
        );
        mat.emissive = Color::rgba(colour.red, colour.green, colour.blue, colour.alpha);
    }
}

pub fn main() {
    let animation = Arc::new(RwLock::new(current_config()));

    let driver = DynDriver::new(animation.clone(), NullBackend);

    App::new()
        .insert_resource(State { driver })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_flycam::PlayerPlugin)
        .add_plugin(Lol::<_, "Animation">(animation.clone()))
        .add_startup_system(setup)
        .add_system(update_driver_system)
        .run();
}
