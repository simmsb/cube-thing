use crate::{
    animation::Animation,
    frame::Frame,
    sdf::{render_sdf, MultiUnion},
};
use nalgebra::Vector3;
use palette::LinSrgba;
use rand::Rng;
use rapier3d::prelude::*;
use sdfu::SDF;

use super::utils::{random_colour, random_rotation};

pub struct Bounce {
    ip: IntegrationParameters,
    pp: PhysicsPipeline,
    im: IslandManager,
    bp: BroadPhase,
    np: NarrowPhase,
    rbs: RigidBodySet,
    cs: ColliderSet,
    ijs: ImpulseJointSet,
    mbjs: MultibodyJointSet,
    ccd: CCDSolver,
    bh: Vec<Ball>,
    sdf_cache: Vec<sdfu::mods::Translate<Vector3<f32>, sdfu::Sphere<f32>>>,
}

#[cfg(feature = "visual")]
impl bevy_inspector_egui::Inspectable for Bounce {
    type Attributes = <() as bevy_inspector_egui::Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        <() as bevy_inspector_egui::Inspectable>::ui(&mut (), ui, options, context)
    }
}

impl std::fmt::Debug for Bounce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bounce").finish()
    }
}

struct Ball {
    handle: RigidBodyHandle,
    colour: LinSrgba,
}

impl Ball {
    fn new(handle: RigidBodyHandle) -> Self {
        Self {
            handle,
            colour: random_colour(1.0, 1.0, 1.0),
        }
    }
}

impl Default for Bounce {
    fn default() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let collider = ColliderBuilder::cuboid(8.0, 8.0, 0.1)
            .translation(vector![0.0, 0.0, 0.0])
            .build();
        collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(8.0, 8.0, 0.1)
            .translation(vector![0.0, 0.0, 8.0])
            .build();
        collider_set.insert(collider);

        let collider = ColliderBuilder::cuboid(0.1, 8.0, 8.0)
            .translation(vector![0.0, 0.0, 0.0])
            .build();
        collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(0.1, 8.0, 8.0)
            .translation(vector![8.0, 0.0, 0.0])
            .build();
        collider_set.insert(collider);

        let collider = ColliderBuilder::cuboid(8.0, 0.1, 8.0)
            .translation(vector![0.0, 0.0, 0.0])
            .build();
        collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(8.0, 0.1, 8.0)
            .translation(vector![0.0, 8.0, 8.0])
            .build();
        collider_set.insert(collider);

        let mut ball_handles = vec![];
        let sdf_cache = vec![];

        let mut rng = rand::thread_rng();

        for _ in 0..rng.gen_range(1..4u8) {
            let initial_vel = vector![0.0, 7.0, 0.0];

            let rotation = random_rotation();

            let initial_position = vector![
                rng.gen_range(1.0..7.0),
                rng.gen_range(1.0..7.0),
                rng.gen_range(1.0..7.0)
            ];

            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(initial_position)
                .linvel(rotation * initial_vel)
                // .linvel(vector![4.0, 2.0, 3.0])
                .build();

            let collider = ColliderBuilder::ball(0.5)
                .restitution(1.0)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .friction(0.0)
                .friction_combine_rule(CoefficientCombineRule::Min)
                .build();
            let ball_body_handle = rigid_body_set.insert(rigid_body);
            collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);
            ball_handles.push(Ball::new(ball_body_handle));
        }

        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        Self {
            ip: integration_parameters,
            pp: physics_pipeline,
            im: island_manager,
            bp: broad_phase,
            np: narrow_phase,
            rbs: rigid_body_set,
            cs: collider_set,
            ijs: impulse_joint_set,
            mbjs: multibody_joint_set,
            ccd: ccd_solver,
            bh: ball_handles,
            sdf_cache,
        }
    }
}

impl Animation for Bounce {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.pp.step(
            &vector![0.0, -9.81, 0.0],
            &self.ip,
            &mut self.im,
            &mut self.bp,
            &mut self.np,
            &mut self.rbs,
            &mut self.cs,
            &mut self.ijs,
            &mut self.mbjs,
            &mut self.ccd,
            &(),
            &(),
        );

        self.sdf_cache.clear();

        for ball in &self.bh {
            let ball_pos = *self.rbs[ball.handle].translation();

            let linvel = *self.rbs[ball.handle].linvel();

            if linvel.magnitude() > 14.0 {
                let new_linvel = linvel.normalize().scale(14.0);
                self.rbs[ball.handle].set_linvel(new_linvel, false);
            }

            let sdf = sdfu::Sphere::new(0.5, ball.colour).translate(ball_pos);

            self.sdf_cache.push(sdf);
        }

        let union = MultiUnion::hard(&self.sdf_cache);

        render_sdf(union, frame);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
