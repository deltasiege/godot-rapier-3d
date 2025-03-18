use super::lookup::LookupTable;
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
            actions: Vec::new(),
            callbacks: Vec::new(),
            // snapshot_buffer: HashMap::new(),
            // snapshot_buffer_max_len: 1000,
            state,
            lookup_table: LookupTable::new(),
        }
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

    pub fn add_callback<F: FnMut(&mut PhysicsState, &RunState) + 'static>(&mut self, callback: F) {
        self.callbacks.push(Box::new(callback));
    }

    pub fn step(&mut self) {
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

        // Remove old snapshots if buffer is full
        // while self.snapshot_buffer.len() > self.snapshot_buffer_max_len {
        //     self.remove_oldest_snapshot_from_buffer();
        // }

        // self.save_current_snapshot_to_buffer(); // Save the current snapshot to the buffer UP TO

        self.state.time += self.physics.integration_parameters.dt as f32;
        self.state.timestep_id += 1;
    }

    /// Retrieve either the current or a buffered snapshot
    pub fn get_snapshot(&mut self, timestep_id: Option<i64>) -> Option<Vec<u8>> {
        match timestep_id {
            None => self.get_current_snapshot(),
            Some(_) => self.get_current_snapshot(), // TODO
                                                    // Some(timestep_id) => self.get_buffered_snapshot(timestep_id),
        }
    }

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

    pub fn get_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.physics.bodies.len(),
            self.physics.colliders.len(),
            self.physics.impulse_joints.len(),
            self.physics.multibody_joints.multibodies().count(),
        )
    }
}
