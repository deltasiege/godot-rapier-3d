use crate::utils::{init_logger, LogLevel};
use crate::{ActionQueue, GR3DPhysicsPipeline, Lookups};
use godot::builtin::PackedByteArray;
use godot::engine::{Engine, IObject, Object};
use godot::prelude::*;

pub const ENGINE_SINGLETON_NAME: &str = "Rapier3DEngine";

pub fn register_engine() {
    init_logger();
    Engine::singleton().register_singleton(
        StringName::from(ENGINE_SINGLETON_NAME),
        GR3DEngineSingleton::new_alloc().upcast(),
    );
}

pub fn unregister_engine() {
    let mut engine = Engine::singleton();
    let singleton_name = StringName::from(ENGINE_SINGLETON_NAME);

    let singleton = engine
        .get_singleton(singleton_name.clone())
        .expect("cannot retrieve the singleton");

    engine.unregister_singleton(singleton_name);
    singleton.free();
}

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DEngineSingleton {
    pub lookups: Lookups,
    pub pipeline: GR3DPhysicsPipeline,
    pub action_queue: ActionQueue,
    pub gizmo_iids: Vec<i64>, // Remembered so that gizmos can be removed
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DEngineSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            lookups: Lookups::new(),
            pipeline: GR3DPhysicsPipeline::new(),
            action_queue: ActionQueue::new(),
            gizmo_iids: Vec::new(),
            base,
        }
    }
}

#[godot_api]
impl GR3DEngineSingleton {
    #[func]
    // Advances the physics simulation 1 step
    pub fn step(&mut self) {
        self.action_queue.push_step_action();
    }

    #[func]
    pub fn get_state(&self) -> PackedByteArray {
        match self.pipeline.state.pack() {
            Ok(vec) => {
                let slice = vec.as_slice();
                PackedByteArray::from(slice)
            }
            Err(e) => {
                log::error!("Could not get state: {:?}", e);
                PackedByteArray::new()
            }
        }
    }

    // TODO will need orchestration?
    #[func]
    pub fn set_state(&mut self, data: PackedByteArray) {
        let slice = data.as_slice();
        match self.pipeline.state.unpack(slice) {
            Ok(state) => {
                self.pipeline.state = state;
                self.pipeline.sync_all_g2r(&self.lookups);
            }
            Err(e) => {
                log::error!("Could not set state: {:?}", e);
                return;
            }
        };
    }

    #[func]
    // Process physics objects (must be done in determinstic order)
    pub fn _process(&mut self) {
        self.action_queue
            ._process(&mut self.pipeline, &mut self.lookups);
    }

    #[func]
    pub fn print_debug_info(&self) {
        log::debug!(
            "Lookups: {:#?}\nPipeline: {}",
            self.lookups.get_all_cuids(),
            self.pipeline
                .get_debug_info()
                .ok()
                .ok_or("Could not get debug info")
                .unwrap()
        );
    }

    #[func]
    pub fn set_log_level(&mut self, level: LogLevel) {
        crate::utils::set_log_level(level);
    }
}

pub fn get_engine() -> Result<Gd<GR3DEngineSingleton>, &'static str> {
    match Engine::singleton().get_singleton(StringName::from("Rapier3DEngine")) {
        Some(gd_pointer) => match gd_pointer.try_cast::<GR3DEngineSingleton>() {
            Ok(singleton) => Ok(singleton),
            Err(_) => Err("Could not cast to Rapier3DEngine singleton"),
        },
        None => Err("Could not retrieve Rapier3DEngine singleton"),
    }
}
