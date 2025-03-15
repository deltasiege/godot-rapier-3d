use super::debugger::GR3DDebugger;
use super::world::{add_nodes_to_world, configure_nodes, move_nodes, remove_nodes_from_world};
use crate::nodes::{generate_cuid, IRapierObject};
use crate::utils::{init_logger, set_log_level};
use crate::world::state::{pack_snapshot, restore_snapshot};
use crate::World;
use godot::classes::{Engine, IObject, Object};
use godot::prelude::*;

/*
    GR3D singleton exposes functions that Godot can call to get or modify Rapier data
*/

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
    /// Return a binary representation of the current state of the simulation
    pub fn get_snapshot(&mut self) -> PackedByteArray {
        let snapshot = pack_snapshot(&self.world);
        match snapshot {
            Ok(snapshot) => PackedByteArray::from(snapshot.as_slice()),
            Err(e) => {
                log::error!("Failed to get_state: {:?}", e);
                PackedByteArray::from(&[])
            }
        }
    }

    #[func]
    /// Set the state of the simulation to the given binary representation
    pub fn restore_snapshot(&mut self, snapshot: PackedByteArray) {
        match restore_snapshot(&mut self.world, snapshot.to_vec()) {
            Ok(_) => {}
            Err(e) => log::error!("Failed to set_state: {:?}", e),
        }
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
    // Register a node in the Rapier world
    pub fn _add_nodes(&mut self, nodes: Array<Gd<Node3D>>) {
        add_nodes_to_world(nodes, &mut self.world);
    }

    #[func]
    // Unregister a node from the Rapier world
    pub fn _remove_nodes(&mut self, nodes: Array<Gd<Node3D>>) {
        remove_nodes_from_world(nodes, &mut self.world);
    }

    #[func]
    // Move a node that already exists in the Rapier world
    pub fn _move_nodes(&mut self, nodes: Array<Gd<Node3D>>, desired_movement: Vector3) {
        move_nodes(nodes, desired_movement, &mut self.world);
    }

    #[func]
    // Ensure fields / settings of Rapier objects match the corresponding godot properties
    pub fn _configure_nodes(&mut self, nodes: Array<Gd<Node3D>>) {
        configure_nodes(nodes, &mut self.world);
    }

    #[func]
    // Draw lines representing the current state of the world according to Rapier
    pub fn _get_debug_lines(&mut self) -> Array<Array<Variant>> {
        self.debugger.render(&self.world)
    }

    #[func]
    // Create a new unique identifier
    pub fn _create_cuid(&self) -> GString {
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

/*
    WARNING:
    All modifications of world through singleton must go via Godot queue first to ensure determinism!

    i.e. Do not call via Godot node -> singleton -> World and then modify the World directly
    Reads are fine, but writes must go via Godot queue for Godot nodes.
*/
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
