use crate::render::{Animation, Frame};
use crate::sdf::render_sdf;
use rand::Rng;
use rapier3d::prelude::*;
use sdfu::SDF;

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
    bh: RigidBodyHandle,
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

        let mut rng = rand::thread_rng();
        let initial_vel = vector![0.0, 7.0, 0.0];
        let rotation = rapier3d::na::Rotation3::from_euler_angles(
            rng.gen_range(0.0..std::f32::consts::TAU),
            rng.gen_range(0.0..std::f32::consts::TAU),
            rng.gen_range(0.0..std::f32::consts::TAU),
        );

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![4.0, 4.0, 4.0])
            .linvel(rotation * initial_vel)
            .build();

        let collider = ColliderBuilder::ball(0.5)
            .restitution(1.0)
            .restitution_combine_rule(CoefficientCombineRule::Max)
            .friction(0.0)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

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
            bh: ball_body_handle,
        }
    }
}

impl Animation for Bounce {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.pp.step(
            &vector![0.0, 0.0, 0.0],
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

        let ball_pos = *self.rbs[self.bh].translation();

        let linvel = *self.rbs[self.bh].linvel();

        if linvel.magnitude() > 7.0 {
            let new_linvel = linvel.normalize().scale(7.0);
            self.rbs[self.bh].set_linvel(new_linvel, false);
        }

        let sdf = sdfu::Sphere::new(0.3).translate(ball_pos);

        render_sdf(sdf, frame);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
