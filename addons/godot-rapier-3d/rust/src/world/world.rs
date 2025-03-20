use super::buffer::WorldBuffer;
use super::state::{pack_snapshot, restore_snapshot, DeserializedPhysicsSnapshot, PhysicsState};
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
    pub buffer: WorldBuffer,
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
            buffer: WorldBuffer::default(),
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

    pub fn step(&mut self) {
        self.buffer
            .execute_actions(self.state.timestep_id, &mut self.physics);

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

        self.buffer
            .on_world_stepped(self.state.timestep_id, self.get_current_snapshot());
    }

    /// Retrieve either the current or a buffered snapshot
    pub fn get_snapshot(&mut self, timestep_id: Option<i64>) -> Option<Vec<u8>> {
        match timestep_id {
            None => self.get_current_snapshot(),
            Some(timestep_id) => self.buffer.get_physics_state(timestep_id as usize),
        }
    }

    /// Retrieve the current snapshot
    fn get_current_snapshot(&self) -> Option<Vec<u8>> {
        let snapshot = pack_snapshot(self);
        match snapshot {
            Ok(snapshot) => Some(snapshot),
            Err(e) => {
                log::error!("Failed to get current snapshot: {:?}", e);
                None
            }
        }
    }

    /// Rolls this world back to the given snapshot's timestep and
    /// applies the snapshots physics state. Rewrites the physics state
    /// history from that point forward back to the current timestep
    pub fn corrective_rollback(&mut self, snapshot: DeserializedPhysicsSnapshot) {
        if let Some(_target_step) = self.buffer.get_step_mut(snapshot.timestep_id) {
            let current_timestep = self.state.timestep_id.clone();
            let steps_to_resim = current_timestep - snapshot.timestep_id;

            match true {
                _ if snapshot.timestep_id >= current_timestep => {
                    log::error!(
                        "Cannot rollback to a future timestep: {}",
                        snapshot.timestep_id
                    );
                }
                _ if steps_to_resim == 0 => {
                    log::warn!("Corrective rollback to same timestep. No action taken.");
                }
                _ if steps_to_resim > self.buffer.max_len => {
                    log::error!(
                        "Cannot rollback more than the buffer length: {}",
                        self.buffer.max_len
                    );
                }
                _ => {
                    self.state.timestep_id = snapshot.timestep_id;
                    self.buffer.mark_physics_stale_after(snapshot.timestep_id);
                    restore_snapshot(self, snapshot, true);
                    for _ in 0..steps_to_resim {
                        self.step();
                    }
                }
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
