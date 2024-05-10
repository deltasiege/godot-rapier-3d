use crate::log::{ LogLevel, Logger };
use crate::physics_pipeline::GR3DPhysicsPipeline;
use godot::builtin::PackedByteArray;
use godot::engine::Engine;
use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;

pub const ENGINE_SINGLETON_NAME: &str = "Rapier3DEngine";

pub fn register_engine() {
    godot_print!("Registering Rapier3DEngine singleton");
    Engine::singleton().register_singleton(
        StringName::from(ENGINE_SINGLETON_NAME),
        GR3DEngineSingleton::new_alloc().upcast()
    );
}

pub fn unregister_engine() {
    let mut engine = Engine::singleton();
    let singleton_name = StringName::from(ENGINE_SINGLETON_NAME);

    let singleton = engine
        .get_singleton(singleton_name.clone())
        .expect("cannot retrieve the singleton");

    godot_print!("Unregistering Rapier3DEngine singleton");
    engine.unregister_singleton(singleton_name);
    singleton.free();
}

#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DEngineSingleton {
    pub pipeline: GR3DPhysicsPipeline,
    pub gizmo_iids: Vec<i64>, // Remembered so that gizmos can be removed
    pub logger: Logger,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DEngineSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            pipeline: GR3DPhysicsPipeline::new(),
            gizmo_iids: Vec::new(),
            logger: Logger::new(LogLevel::Info),
            base,
        }
    }
}

#[godot_api]
impl GR3DEngineSingleton {
    #[func]
    pub fn step(&mut self) {
        self.pipeline.step();
    }

    #[func]
    pub fn get_state(&self) -> PackedByteArray {
        let vec = self.pipeline.state.pack();
        let slice = vec.as_slice();
        PackedByteArray::from(slice)
    }

    #[func]
    pub fn set_state(&mut self, data: PackedByteArray) {
        let slice = data.as_slice();
        let state = self.pipeline.state.unpack(slice);
        self.pipeline.state = state;
        self.pipeline.sync_all_body_positions();
    }

    #[func]
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.logger.set_level(level);
    }

    #[func]
    pub fn print_debug_info(&self) {
        godot_print!(
            "Rigid body ids: {:?}
Collider ids: {:?}",
            self.pipeline.rigid_body_ids,
            self.pipeline.collider_ids
        );

        for (handle, rb) in self.pipeline.state.rigid_body_set.iter() {
            godot_print!("Rigid body {:?}: {:?}", handle, rb.position());
        }

        for (handle, collider) in self.pipeline.state.collider_set.iter() {
            godot_print!("Collider {:?}: {:?}", handle, collider.position());
        }
    }
}

#[macro_export]
macro_rules! get_engine {
    () => {
        {
            let gd_pointer = match
                godot::engine::Engine::singleton().get_singleton(StringName::from("Rapier3DEngine"))
            {
                Some(gd_pointer) => gd_pointer,
                None => {
                    godot_error!("Could not obtain Rapier3DEngine singleton");
                    return;
                }
            };
        
            let singleton = match gd_pointer.try_cast::<crate::engine::GR3DEngineSingleton>() {
                Ok(singleton) => singleton,
                Err(_) => {
                    godot_error!("Could not cast to Rapier3DEngine singleton");
                    return;
                }
            };
        
            singleton
        }
    };
}
