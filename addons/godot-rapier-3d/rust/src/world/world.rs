use super::lookup::LookupTable;
use super::state::PhysicsState;
use rapier3d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodySet,
};
use rapier3d::geometry::{ColliderSet, DefaultBroadPhase, NarrowPhase};
use rapier3d::math::{Real, Vector};
use rapier3d::pipeline::{PhysicsHooks, PhysicsPipeline, QueryPipeline};
use rapier3d::prelude::{Collider, ColliderHandle, RigidBody, RigidBodyHandle};

pub struct RunState {
    pub timestep_id: usize,
    pub time: f32,
}

impl Default for RunState {
    fn default() -> Self {
        Self::new()
    }
}

impl RunState {
    pub fn new() -> Self {
        Self {
            timestep_id: 0,
            time: 0.0,
        }
    }
}

pub struct World {
    pub physics: PhysicsState,
    max_steps: usize,
    actions: Actions,
    callbacks: Callbacks,
    pub state: RunState,
    pub lookup_table: LookupTable,
}

type Actions = Vec<Box<dyn FnMut(&mut PhysicsState, &RunState)>>; // Actions are called before stepping and then deleted each step
type Callbacks = Vec<Box<dyn FnMut(&mut PhysicsState, &RunState)>>; // Callbacks are called after stepping every step

impl World {
    pub fn new_empty() -> Self {
        let physics = PhysicsState::new();
        let state = RunState::new();

        Self {
            physics,
            max_steps: 1000,
            actions: Vec::new(),
            callbacks: Vec::new(),
            state,
            lookup_table: LookupTable::new(),
        }
    }

    pub fn new(
        bodies: RigidBodySet,
        colliders: ColliderSet,
        impulse_joints: ImpulseJointSet,
        multibody_joints: MultibodyJointSet,
        lookup_table: LookupTable,
    ) -> Self {
        let mut res = Self::new_empty();
        res.set_world(bodies, colliders, impulse_joints, multibody_joints);
        res.lookup_table = lookup_table;
        res
    }

    pub fn set_max_steps(&mut self, max_steps: usize) {
        self.max_steps = max_steps
    }

    pub fn integration_parameters_mut(&mut self) -> &mut IntegrationParameters {
        &mut self.physics.integration_parameters
    }

    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }

    pub fn clear_callbacks(&mut self) {
        self.callbacks.clear();
    }

    pub fn physics_state_mut(&mut self) -> &mut PhysicsState {
        &mut self.physics
    }

    pub fn set_world(
        &mut self,
        bodies: RigidBodySet,
        colliders: ColliderSet,
        impulse_joints: ImpulseJointSet,
        multibody_joints: MultibodyJointSet,
    ) {
        self.set_world_with_params(
            bodies,
            colliders,
            impulse_joints,
            multibody_joints,
            Vector::y() * -9.81,
            (),
        )
    }

    pub fn set_world_with_params(
        &mut self,
        bodies: RigidBodySet,
        colliders: ColliderSet,
        impulse_joints: ImpulseJointSet,
        multibody_joints: MultibodyJointSet,
        gravity: Vector<Real>,
        hooks: impl PhysicsHooks + 'static,
    ) {
        // println!("Num bodies: {}", bodies.len());
        // println!("Num impulse_joints: {}", impulse_joints.len());
        self.physics.gravity = gravity;
        self.physics.bodies = bodies;
        self.physics.colliders = colliders;
        self.physics.impulse_joints = impulse_joints;
        self.physics.multibody_joints = multibody_joints;
        self.physics.hooks = Box::new(hooks);

        self.physics.islands = IslandManager::new();
        self.physics.broad_phase = DefaultBroadPhase::new();
        self.physics.narrow_phase = NarrowPhase::new();
        self.state.timestep_id = 0;
        self.state.time = 0.0;
        self.physics.ccd_solver = CCDSolver::new();
        self.physics.query_pipeline = QueryPipeline::new();
        self.physics.pipeline = PhysicsPipeline::new();
        self.physics.pipeline.counters.enable();
    }

    pub fn add_bodies(&mut self, bodies: Vec<impl Into<RigidBody>>) {
        for body in bodies {
            self.physics.bodies.insert(body.into());
        }
    }

    pub fn add_colliders(&mut self, colliders: Vec<impl Into<Collider>>) {
        for collider in colliders {
            self.physics.colliders.insert(collider.into());
        }
    }

    pub fn remove_bodies(&mut self, handles: Vec<RigidBodyHandle>) {
        for handle in handles {
            self.physics.bodies.remove(
                handle,
                &mut self.physics.islands,
                &mut self.physics.colliders,
                &mut self.physics.impulse_joints,
                &mut self.physics.multibody_joints,
                false,
            );

            self.lookup_table.remove_by_handle(&handle.into_raw_parts());
        }
    }

    pub fn remove_colliders(&mut self, handles: Vec<ColliderHandle>, wake_up: bool) {
        for handle in handles {
            self.physics.colliders.remove(
                handle,
                &mut self.physics.islands,
                &mut self.physics.bodies,
                wake_up,
            );

            self.lookup_table.remove_by_handle(&handle.into_raw_parts());
        }
    }

    pub fn add_action<F: FnMut(&mut PhysicsState, &RunState) + 'static>(&mut self, callback: F) {
        self.actions.push(Box::new(callback));
    }

    pub fn add_callback<F: FnMut(&mut PhysicsState, &RunState) + 'static>(&mut self, callback: F) {
        self.callbacks.push(Box::new(callback));
    }

    pub fn step(&mut self) {
        for f in &mut self.actions {
            f(&mut self.physics, &self.state);
        }
        self.clear_actions();

        self.physics.pipeline.step(
            &self.physics.gravity,
            &self.physics.integration_parameters,
            &mut self.physics.islands,
            &mut self.physics.broad_phase,
            &mut self.physics.narrow_phase,
            &mut self.physics.bodies,
            &mut self.physics.colliders,
            &mut self.physics.impulse_joints,
            &mut self.physics.multibody_joints,
            &mut self.physics.ccd_solver,
            Some(&mut self.physics.query_pipeline),
            &*self.physics.hooks,
            &(),
        );

        for f in &mut self.callbacks {
            f(&mut self.physics, &self.state);
        }

        self.state.time += self.physics.integration_parameters.dt as f32;
        self.state.timestep_id += 1;
    }

    pub fn run(&mut self) {
        for _ in 0..self.max_steps {
            self.step();
        }
    }

    pub fn get_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.physics.bodies.len(),
            self.physics.colliders.len(),
            self.physics.impulse_joints.len(),
            self.physics.multibody_joints.multibodies().count(),
        )
    }

    pub fn print_state(&self) {
        let total = self.physics.bodies.len()
            + self.physics.colliders.len()
            + self.physics.impulse_joints.len()
            + self.physics.multibody_joints.multibodies().count();
        log::info!("Total: {}", total);
        log::info!("|_ bodies: {}", self.physics.bodies.len());
        log::info!("|_ colliders: {}", self.physics.colliders.len());
        log::info!("|_ impulse_joints: {}", self.physics.impulse_joints.len());
        log::info!(
            "|_ multibody_joints: {}",
            self.physics.multibody_joints.multibodies().count()
        );
    }
}
