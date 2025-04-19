use crate::interface::GR3D;
use crate::types::*;
use crate::world::{step, WorldSnapshot};

pub struct RollbackState {
    pub count: u64, // Total number of rollbacks performed. Used to identify rollbacks in logs.
    pub invalid_ticks: Vec<Tick>, // List of ticks where rollbacks are required. Cleared on every rollback.
}

impl RollbackState {
    pub fn new() -> Self {
        Self {
            count: 0,
            invalid_ticks: Vec::new(),
        }
    }
}

/// Occurs every Godot physics_process.
/// Retrieve the earliest rollback_flag if any, and resimulate the world from that tick.
pub fn process_rollbacks(gr3d: &mut GR3D) {
    let earliest_tick = match gr3d.rollback_state.invalid_ticks.iter().min() {
        Some(&tick) => tick,
        None => return,
    };

    log::trace!("Rollback required at tick: {}", earliest_tick);
    resimulate_from_tick(gr3d, earliest_tick);
    gr3d.rollback_state.invalid_ticks.clear();
}

/// Resimulate the world from a given tick.
pub fn resimulate_from_tick(gr3d: &mut GR3D, tick: Tick) {
    let instant = std::time::Instant::now();

    log::trace!(
        "Rollback #{} START. Tick range: {} -> {}",
        gr3d.rollback_state.count,
        tick,
        gr3d.world.time.tick
    );
    {
        let _ = try_resimulate_from_tick(gr3d, tick);
    }
    log::trace!(
        "Rollback #{} END [{} μs].",
        gr3d.rollback_state.count,
        instant.elapsed().as_micros()
    );
    gr3d.rollback_state.count += 1;
}

fn try_resimulate_from_tick(gr3d: &mut GR3D, tick: Tick) -> Result<(), ()> {
    let current_tick = gr3d.world.time.tick.clone();
    let ser_snapshot = match gr3d.network.local_peer.world_snapshots.get(&tick) {
        Some(ser_snapshot) => ser_snapshot,
        None => {
            log::error!("No snapshot found for tick: {}", tick);
            return Err(());
        }
    };

    let snapshot = match WorldSnapshot::try_from_bytes(&ser_snapshot) {
        Some(snapshot) => snapshot,
        None => {
            log::error!("Failed to decode snapshot for tick: {}", tick);
            return Err(());
        }
    };

    gr3d.world.restore_snapshot(snapshot, true);

    let steps = current_tick - tick;
    log::trace!("Stepping {} times", steps);
    step(gr3d, steps);

    Ok(())
}
