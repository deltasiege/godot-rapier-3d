use crate::interface::GR3D;
use crate::world::*;

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
    pub fn step(&mut self) -> WorldSnapshot {
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

        let snap = self.save_de_snapshot();

        self.time.secs += self.physics.integration_parameters.dt as f32;
        self.time.tick += 1;
        log::trace!("Tick counter advanced to: {}", self.time.tick);

        snap
    }

    /// Get a serialized version of the current world snapshot
    pub fn save_snapshot(&self) -> Option<Vec<u8>> {
        log::trace!("Encoding snapshot of world at tick: {}", self.time.tick);
        self.save_de_snapshot().try_to_bytes()
    }

    /// Get a snapshot of the current world state
    pub fn save_de_snapshot(&self) -> WorldSnapshot {
        log::trace!("Getting snapshot of world at tick: {}", self.time.tick);
        WorldSnapshot::from_world(self)
    }

    /// Load a serialized WorldSnapshot
    pub fn load_snapshot(&mut self, snapshot: &Vec<u8>, overwrite_tick: bool) {
        log::trace!("Decoding snapshot of length: {}", snapshot.len());
        if let Some(snapshot) = WorldSnapshot::try_from_bytes(&snapshot) {
            self.load_de_snapshot(snapshot, overwrite_tick);
        }
    }

    /// Load a WorldSnapshot, optionally overwriting the tick
    pub fn load_de_snapshot(&mut self, snapshot: WorldSnapshot, overwrite_tick: bool) {
        let op = match overwrite_tick {
            true => "Restoring",
            false => "Rolling back",
        };
        log::trace!("{} world: {} -> {}", op, self.time.tick, snapshot.tick);
        snapshot.apply_to_world(self, overwrite_tick);
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

pub fn step(gr3d: &mut GR3D, count: u64) {
    for _ in 0..count {
        let tick = gr3d.world.time.tick.clone();
        log::trace!("Executing tick: {}", tick);

        let local_peer_id = gr3d.network.local_peer.get_peer_id().unwrap_or(-1);
        gr3d.world
            .node_db
            .process_node_tick_functions(local_peer_id);
        process_rapier_actions(gr3d);

        let snapshot = gr3d.world.step();
        gr3d.network
            .local_peer
            .record_world_snapshot(tick, snapshot);

        log::trace!("Tick finished: {}", tick);
    }
}
