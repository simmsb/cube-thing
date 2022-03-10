use crate::render::{Animation, Frame};
use crate::sdf::{render_sdf, MultiUnion};
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
    js: JointSet,
    ccd: CCDSolver,
    bh: RigidBodyHandle, // ball_handle:
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

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![4.0, 4.0, 4.0])
            .linvel(vector![4.0, 5.0, 2.0])
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
        let joint_set = JointSet::new();
        let ccd_solver = CCDSolver::new();

        Self {
            ip: integration_parameters,
            pp: physics_pipeline,
            im: island_manager,
            bp: broad_phase,
            np: narrow_phase,
            rbs: rigid_body_set,
            cs: collider_set,
            js: joint_set,
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
            &mut self.js,
            &mut self.ccd,
            &(),
            &(),
        );

        let ball_pos = *self.rbs[self.bh].translation();

        let sdf = sdfu::Sphere::new(0.3)
            .translate(ultraviolet::Vec3::new(ball_pos.x, ball_pos.y, ball_pos.z));

        render_sdf(sdf, frame);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
