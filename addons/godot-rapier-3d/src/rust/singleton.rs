use crate::physics_pipeline::RapierPhysicsPipeline;
use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Rapier3DSingleton {
    pub pipeline: RapierPhysicsPipeline,
    base: Base<Object>,
}

#[godot_api]
impl IObject for Rapier3DSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            pipeline: RapierPhysicsPipeline::new(),
            base,
        }
    }
}

#[godot_api]
impl Rapier3DSingleton {
    #[func]
    pub fn step(&mut self) {
        self.pipeline.step();
    }

    #[func]
    pub fn print_rigid_body_ids(&self) {
        godot_print!("Rigid body ids: {:?}", self.pipeline.rigid_body_ids);
    }
}
