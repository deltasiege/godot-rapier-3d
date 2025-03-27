use super::debugger::GR3DDebugger;
use crate::nodes::{generate_cuid, IRapierObject};
use crate::utils::extract_from_dict;
use crate::utils::{init_logger, set_log_level};
use crate::world::buffer::{ingest_local_action, Action};
use crate::world::state::{restore_snapshot, unpack_snapshot};
use crate::world::Operation;
use crate::World;
use godot::classes::file_access::CompressionMode;
use godot::classes::{Engine, IObject, Object};
use godot::prelude::*;

/*
    GR3D singleton exposes functions that Godot can call to get or modify Rapier data
*/

/// Use the GR3D singleton to interact with the Rapier physics engine
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3D {
    pub world: World,
    debugger: GR3DDebugger,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3D {
    fn init(base: Base<Object>) -> Self {
        Self {
            world: World::new_empty(),
            debugger: GR3DDebugger::new(),
            base,
        }
    }
}

#[godot_api]
impl GR3D {
    #[func]
    /// Advance the simulation by the given number of steps
    pub fn step(&mut self, count: i64) {
        for _ in 0..count {
            self.world.step();
        }
    }

    #[func]
    /// Returns a past world state as a snapshot. The timestep_id must still be in the world buffer.
    pub fn get_snapshot(&mut self, timestep_id: i64) -> PackedByteArray {
        match self.world.get_snapshot(Some(timestep_id)) {
            Some(snapshot) => {
                match PackedByteArray::from(snapshot.as_slice()).compress(CompressionMode::BROTLI) {
                    Ok(compressed) => compressed,
                    Err(e) => {
                        log::error!("Failed to compress snapshot: {:?}", e);
                        PackedByteArray::from(&[])
                    }
                }
            }
            None => PackedByteArray::from(&[]),
        }
    }

    #[func]
    /// Returns the current world state as a snapshot
    pub fn save_snapshot(&mut self) -> PackedByteArray {
        match self.world.get_snapshot(None) {
            Some(snapshot) => PackedByteArray::from(snapshot.as_slice()),
            None => PackedByteArray::from(&[]),
        }
    }

    #[func]
    /// Overwrite the current state of the simulation to match the given snapshot
    pub fn restore_snapshot(&mut self, snapshot: PackedByteArray) {
        let bytes = unpack_snapshot(snapshot.to_vec());
        if let Some(snapshot) = bytes {
            restore_snapshot(&mut self.world, snapshot, false);
        }
    }

    #[func]
    /// Rollback the simulation to the given timestep
    /// Optionally replicate the actions in the buffer up until the current timestep
    /// Then roll-forward the simulation to get back to the current timestep, re-executing all actions in the buffer along the way
    /// Actions should be of the form: { "cuid": "unique_id", "node": Node3D, "operation": Operation, "data": Dictionary, "replicate": bool }
    pub fn rollback(
        &mut self,
        timestep_id: i64,
        actions_to_add: Array<Dictionary>,
        snapshot: PackedByteArray,
    ) {
        self.world.corrective_rollback(
            timestep_id as usize,
            actions_from_array(actions_to_add),
            unpack_snapshot(snapshot.to_vec()),
        );
    }

    #[func]
    /// Get the current count of all objects registered in the simulation
    pub fn get_counts(&self) -> Dictionary {
        let counts = self.world.get_counts();
        let mut dict = Dictionary::new();
        dict.set("bodies", counts.0 as i64);
        dict.set("colliders", counts.1 as i64);
        dict.set("joints", counts.2 as i64);
        dict.set("multibodies", counts.3 as i64);
        dict
    }

    #[func]
    /// Returns the current tick
    pub fn get_tick(&self) -> i64 {
        self.world.state.timestep_id as i64
    }

    #[func]
    /// Returns the current timestamp
    pub fn get_time(&self) -> f64 {
        self.world.state.time as f64
    }

    #[func]
    /// TODO replace wiht "ingest serialized actions"
    pub fn get_actions(&self) -> Array<Dictionary> {
        let timestep_id = self.world.state.timestep_id;
        let mut result = Array::new();

        if let Some(actions) = self.world.buffer.get_actions(timestep_id - 1) {
            for action in actions {
                let mut dict = Dictionary::new();
                dict.set("cuid", action.cuid.to_variant());

                if let Some(handle) = action.handle {
                    dict.set("handle", [handle.0, handle.1].to_variant());
                }

                dict.set("node", action.node.to_variant());
                dict.set("operation", action.operation.to_variant());
                dict.set("data", action.data.to_variant());
                result.push(&dict);
            }
        }
        result
    }

    #[func]
    /// Consume a local action and apply it to the world immediately
    pub fn _ingest_local_action(&mut self, node: Gd<Node>, operation: Operation, data: Dictionary) {
        ingest_local_action(node, operation, data, &mut self.world);
    }

    #[func]
    // Draw lines representing the current state of the world according to Rapier
    pub fn _get_debug_lines(&mut self) -> Array<Array<Variant>> {
        self.debugger.render(&self.world)
    }

    #[func]
    /// Create a new unique identifier
    /// Should be used when spawning new Rapier physics nodes via code
    /// to ensure that the CUID is unique on the spawned node (via set_uid)
    pub fn create_cuid(&self) -> GString {
        generate_cuid()
    }

    #[signal]
    pub fn draw_line_request(a: Vector3, b: Vector3, color: Color);
}

pub const NAME: &str = "GR3D";

pub fn register() {
    init_logger();
    set_log_level(crate::utils::LogLevel::Debug);

    Engine::singleton().register_singleton(NAME, &GR3D::new_alloc());
}

pub fn unregister() {
    let mut engine = Engine::singleton();
    if let Some(my_singleton) = engine.get_singleton(NAME) {
        engine.unregister_singleton(NAME);
        my_singleton.free();
    } else {
        log::error!("Failed to get {} singleton", NAME);
    }
}

pub fn get_singleton() -> Option<Gd<GR3D>> {
    match Engine::singleton().get_singleton(NAME) {
        Some(singleton) => Some(singleton.cast::<GR3D>()),
        None => {
            log::error!("Failed to get {} singleton", NAME);
            None
        }
    }
}

pub fn get_tree(node: &impl IRapierObject) -> Option<Gd<SceneTree>> {
    node.base().get_tree()
}

// Convert Array of Dictionaries to Vec of Actions
fn actions_from_array(actions: Array<Dictionary>) -> Option<Vec<Action>> {
    let result: Vec<Action> = actions
        .iter_shared()
        .filter_map(|data| action_from_dict(data))
        .collect();

    if result.len() == 0 {
        None
    } else {
        Some(result)
    }
}

// Convert Dictionary to Action
fn action_from_dict(dict: Dictionary) -> Option<Action> {
    let cuid = GString::from(dict.get("cuid")?.to_string());
    let handle = extract_handle_from_action_dict(&dict)?;
    let node = extract_from_dict(&dict, "node", true)?;
    let operation = extract_from_dict(&dict, "operation", true)?;
    let data = extract_from_dict(&dict, "data", true)?;
    Some(Action::new(cuid, Some(handle), node, operation, data))
}

fn extract_handle_from_action_dict(dict: &Dictionary) -> Option<(u32, u32)> {
    let raw_handle = extract_from_dict::<Array<i64>>(dict, "handle", true)?;
    Some((raw_handle.at(0) as u32, raw_handle.at(1) as u32))
}
