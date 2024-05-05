use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;
use rapier3d::math::Vector as RVector;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Rapier3DSingleton {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: RVector<Real>,
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
    base: Base<Object>,
}

#[godot_api]
impl IObject for Rapier3DSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: RVector::new(0.0, -9.81, 0.0),
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
impl Rapier3DSingleton {
    #[func]
    pub fn say_hi(&self) {
        godot_print!("Hi from singleton!");
    }

    #[func]
    pub fn step(&mut self) {
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
    }
}
