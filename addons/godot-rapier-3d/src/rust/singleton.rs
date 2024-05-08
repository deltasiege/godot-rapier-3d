use crate::physics_pipeline::RapierPhysicsPipeline;
use godot::builtin::PackedByteArray;
use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Rapier3DEngineSingleton {
    pub pipeline: RapierPhysicsPipeline,
    pub gizmo_iids: Vec<i64>, // Remembered so that gizmos can be removed
    base: Base<Object>,
}

#[godot_api]
impl IObject for Rapier3DEngineSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            pipeline: RapierPhysicsPipeline::new(),
            gizmo_iids: Vec::new(),
            base,
        }
    }
}

#[godot_api]
impl Rapier3DEngineSingleton {
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
