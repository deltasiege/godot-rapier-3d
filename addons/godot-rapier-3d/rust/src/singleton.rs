use crate::physics_pipeline::GR3DPhysicsPipeline;
use godot::builtin::PackedByteArray;
use godot::engine::Engine;
use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;

pub fn register_engine() {
    godot_print!("Registering Rapier3DEngine singleton");
    Engine::singleton().register_singleton(
        crate::utils::get_engine_singleton_name(),
        GR3DEngineSingleton::new_alloc().upcast(),
    );
}

pub fn unregister_engine() {
    let mut engine = Engine::singleton();
    let singleton_name = crate::utils::get_engine_singleton_name();

    let singleton = engine
        .get_singleton(singleton_name.clone())
        .expect("cannot retrieve the singleton");

    godot_print!("Unregistering Rapier3DEngine singleton");
    engine.unregister_singleton(singleton_name);
    singleton.free();
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct GR3DEngineSingleton {
    pub pipeline: GR3DPhysicsPipeline,
    pub gizmo_iids: Vec<i64>, // Remembered so that gizmos can be removed
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DEngineSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            pipeline: GR3DPhysicsPipeline::new(),
            gizmo_iids: Vec::new(),
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
