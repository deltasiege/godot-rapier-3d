use crate::world::{DebugVisualizer, NodeDatabase, PhysicsState, WorldSnapshot};

pub struct World {
    pub time: TimeState,
    pub physics: PhysicsState,
    pub node_db: NodeDatabase,
    pub debugger: DebugVisualizer,
}

impl World {
    pub fn new() -> Self {
        Self {
            time: TimeState::new(),
            physics: PhysicsState::new(),
            node_db: NodeDatabase::new(),
            debugger: DebugVisualizer::new(),
        }
    }

    /// Advance the simulation by one step
    /// Return the next tick and the resulting snapshot
    pub fn step(&mut self) -> (usize, Option<Vec<u8>>) {
        log::trace!("Stepping world at tick: {}", self.time.tick);

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
        log::trace!("Taking snapshot of world at tick: {}", self.time.tick);
        WorldSnapshot::from_world(self).try_to_bytes()
    }

    /// Overwrite the current state of the given world to the given snapshot state
    pub fn restore_snapshot(&mut self, snapshot: WorldSnapshot, overwrite_tick: bool) {
        let op = match overwrite_tick {
            true => "Restoring",
            false => "Rolling back",
        };
        log::trace!("{} world: {} -> {}", op, self.time.tick, snapshot.tick);
        snapshot.apply_to_world(self, overwrite_tick)
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
