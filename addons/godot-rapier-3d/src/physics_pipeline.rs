use crate::collider::RapierCollider3D;
use crate::rigid_body::RapierRigidBody3D;
use godot::engine::IRefCounted;
use godot::engine::RefCounted;
use godot::prelude::*;
use rapier3d::na;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct RapierPhysicsPipeline {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: na::SVector<f32, 3>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RapierPhysicsPipeline {
    fn init(base: Base<RefCounted>) -> Self {
        godot_print!("RapierPhysicsPipeline::init()");

        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        // let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
        // collider_set.insert(collider);

        // let rigid_body = RigidBodyBuilder::dynamic()
        //     .translation(vector![0.0, 10.0, 0.0])
        //     .build();
        // let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        // let ball_body_handle = rigid_body_set.insert(rigid_body);
        // collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

        Self {
            // rigid_body_set: RigidBodySet::new(),
            // collider_set: ColliderSet::new(),
            rigid_body_set,
            collider_set,
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: (),
            base,
        }
    }
}

#[godot_api]
impl RapierPhysicsPipeline {
    #[signal]
    fn physics_step();

    #[func]
    fn step(&mut self) {
        godot_print!("RapierPhysicsPipeline::step()");

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );

        // let ball_body = &self.rigid_body_set[self.ball_body_handle];

        // godot_print!("Ball altitude: {}", ball_body.translation().y);

        self.base_mut().emit_signal("stepped".into(), &[]);
    }

    #[func]
    fn add_rigid_body(&mut self, mut rigid_body: Gd<RapierRigidBody3D>) {
        let mut rb = rigid_body.bind_mut();
        let handle = self.rigid_body_set.insert(rb.rigid_body.clone());
        rb.handle = handle;
    }

    #[func]
    fn remove_rigid_body(&mut self, rigid_body: Gd<RapierRigidBody3D>) {
        let rb = rigid_body.bind();
        self.rigid_body_set.remove(
            rb.handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            false,
        );
    }

    #[func]
    fn count_rigid_bodies(&self) -> i64 {
        self.rigid_body_set.len() as i64
    }

    // TODO remove me
    #[func]
    fn print_rigid_bodies(&mut self) {
        self.rigid_body_set.iter().for_each(|(handle, rigid_body)| {
            godot_print!("Handle: {:?} | Mass: {:?}", handle, rigid_body.mass());
        });
    }

    #[func]
    fn add_collider_with_parent(
        &mut self,
        mut collider: Gd<RapierCollider3D>,
        parent: Gd<RapierRigidBody3D>,
    ) {
        let mut col = collider.bind_mut();
        let rb = parent.bind();
        let handle = self.collider_set.insert_with_parent(
            col.collider.clone(),
            rb.handle,
            &mut self.rigid_body_set,
        );
        col.handle = handle;
        col.parent = Some(rb.handle);
    }

    #[func]
    fn remove_collider(&mut self, collider: Gd<RapierCollider3D>) {
        let col = collider.bind();
        self.collider_set.remove(
            col.handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            false,
        );
    }

    #[func]
    fn count_colliders(&self) -> i64 {
        self.collider_set.len() as i64
    }
}
