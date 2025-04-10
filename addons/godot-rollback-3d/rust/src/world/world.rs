use godot::builtin::PackedByteArray;

use crate::{interface::GR3D, world::*};

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

    pub fn reset(&mut self) {
        log::trace!("Resetting world to empty state");
        self.time = TimeState::new();
        self.physics = PhysicsState::new();
        self.node_db = NodeDatabase::new();
        self.debugger = DebugVisualizer::new();
    }

    /// Advance the simulation by one step
    /// Return the resulting world snapshot after stepping
    pub fn step(&mut self) -> Option<Vec<u8>> {
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

        let snap = self.save_snapshot();

        self.time.secs += self.physics.integration_parameters.dt as f32;
        self.time.tick += 1;
        log::trace!("Tick counter advanced to: {}", self.time.tick);

        snap
    }

    /// Retrieve the current snapshot
    pub fn save_snapshot(&self) -> Option<Vec<u8>> {
        log::trace!("Taking snapshot of world at tick: {}", self.time.tick);
        WorldSnapshot::from_world(self).try_to_bytes()
    }

    /// Convert a PackedByteArray into a WorldSnapshot and then restore it
    pub fn load_snapshot(&mut self, snapshot: PackedByteArray) {
        match WorldSnapshot::try_from_bytes(&snapshot.to_vec()) {
            Some(snapshot) => self.restore_snapshot(snapshot, false),
            None => log::error!("Failed to load snapshot from PackedByteArray"),
        }
    }

    /// Overwrite the current state of the world to the given snapshot state
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

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("time", &self.time)
            .field("physics", &self.physics)
            .field("node_db", &self.node_db)
            .finish()
    }
}

pub fn step(gr3d: &mut GR3D, count: i64) {
    for _ in 0..count {
        let tick = gr3d.world.time.tick;
        log::trace!("Executing tick: {}", tick);

        // TODO execute all inputs
        // log::trace!("Applied inputs for tick {}: {:?}", tick, inputs);

        let _resulting_state = gr3d.world.step();

        // TODO add resulting state to local_peer buffer
        log::trace!("Tick finished: {}", tick);
    }
}
