use crate::world::{DebugVisualizer, PhysicsState};

pub struct World {
    pub time: TimeState,
    pub physics: PhysicsState,
    pub debugger: DebugVisualizer,
}

impl World {
    pub fn new_empty() -> Self {
        Self {
            time: TimeState::new(),
            physics: PhysicsState::new(),
            debugger: DebugVisualizer::new(),
        }
    }

    /// Advance the simulation by one step
    /// Return the next tick and the resulting snapshot
    pub fn step(&mut self) -> (usize, Option<Vec<u8>>) {
        log::trace!("Stepping local world at tick: {}", self.time.tick);

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

        self.time.secs += self.physics.integration_parameters.dt as f32;
        self.time.tick += 1;

        let snap = self.take_snapshot();

        (self.time.tick, snap)
    }

    /// Retrieve the current snapshot
    pub fn take_snapshot(&self) -> Option<Vec<u8>> {
        let snapshot = pack_snapshot(self);
        match snapshot {
            Ok(snapshot) => Some(snapshot),
            Err(e) => {
                log::error!("Failed to get current snapshot: {:?}", e);
                None
            }
        }
    }

    /// Overwrite the current state of the given world to the given snapshot state
    pub fn restore_snapshot(
        &mut self,
        snapshot: DeserializedPhysicsSnapshot,
        overwrite_timestep: bool,
    ) {
        if overwrite_timestep {
            world.state.timestep_id = snapshot.timestep_id;
        }

        world.physics.broad_phase = snapshot.broad_phase;
        world.physics.narrow_phase = snapshot.narrow_phase;
        world.physics.islands = snapshot.island_manager;
        world.physics.bodies = snapshot.bodies;
        world.physics.impulse_joints = snapshot.impulse_joints;
        world.physics.multibody_joints = snapshot.multibody_joints;

        // Carefully handle colliders to not overwrite those excluded from snapshots
        for (handle, collider) in snapshot.colliders.iter() {
            if let Some(collider) = world.physics.colliders.get_mut(handle) {
                *collider = collider.clone();
            } else {
                world.physics.colliders.insert(collider.clone());
            }
        }

        world.physics.lookup_table = snapshot.lookup_table;
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

pub struct TimeState {
    pub tick: usize,
    pub secs: f32,
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeState {
    pub fn new() -> Self {
        Self { tick: 0, secs: 0.0 }
    }
}
