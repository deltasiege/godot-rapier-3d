use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::na;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
struct RapierCollider {
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("RapierCollider::init()");

        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
            base,
        }
    }
}

#[godot_api]
impl RapierCollider {
    #[func]
    fn step(&mut self) {
        godot_print!("RapierCollider::step()");

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

        self.base_mut().emit_signal("stepped".into(), &[]);
    }

    #[signal]
    fn physics_step();
}
