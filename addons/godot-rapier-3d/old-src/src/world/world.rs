use crate::utils::get_hash;

use super::state::{pack_snapshot, PhysicsState};
use rapier3d::dynamics::IntegrationParameters;

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
    callbacks: Callbacks,
    pub state: RunState,
}

type Callbacks = Vec<Box<dyn FnMut(&mut PhysicsState, &RunState)>>; // Callbacks are called after stepping every step

impl World {
    pub fn new_empty() -> Self {
        let physics = PhysicsState::new();
        let state = RunState::new();
        Self {
            physics,
            callbacks: Vec::new(),
            state,
        }
    }

    pub fn integration_parameters_mut(&mut self) -> &mut IntegrationParameters {
        &mut self.physics.integration_parameters
    }

    pub fn clear_callbacks(&mut self) {
        self.callbacks.clear();
    }

    pub fn physics_state_mut(&mut self) -> &mut PhysicsState {
        &mut self.physics
    }

    pub fn add_callback<F: FnMut(&mut PhysicsState, &RunState) + 'static>(&mut self, callback: F) {
        self.callbacks.push(Box::new(callback));
    }

    /// Advance the simulation by one step
    /// Return the resulting snapshot and the next tick idx
    pub fn step(&mut self) -> (Option<Vec<u8>>, usize) {
        log::trace!(
            "Stepping local world at timestep: {}",
            self.state.timestep_id
        );

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

        let snap = self.get_current_snapshot();

        (snap, self.state.timestep_id)
    }

    /// Retrieve either the current or a buffered snapshot
    // pub fn get_snapshot(&mut self, timestep_id: Option<i64>) -> Option<Vec<u8>> {
    //     match timestep_id {
    //         None => self.get_current_snapshot(),
    //         Some(timestep_id) => self.buffer.get_physics_state(timestep_id as usize),
    //     }
    // }

    /// Retrieve the current snapshot
    pub fn get_current_snapshot(&self) -> Option<Vec<u8>> {
        let snapshot = pack_snapshot(self);
        match snapshot {
            Ok(snapshot) => Some(snapshot),
            Err(e) => {
                log::error!("Failed to get current snapshot: {:?}", e);
                None
            }
        }
    }

    /// Return the amount of bodies, colliders, impulse joints, and multibody joints in the world
    pub fn get_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.physics.bodies.len(),
            self.physics.colliders.len(),
            self.physics.impulse_joints.len(),
            self.physics.multibody_joints.multibodies().count(),
        )
    }
}
