use crate::objects::Handle;
use cuid2 as _cuid2;
use godot::obj::WithBaseField;
use godot::prelude::*;

pub fn cuid2() -> String {
    _cuid2::create_id()
}

pub fn node_from_instance_id<T>(instance_id: InstanceId) -> Result<Gd<T>, String>
where
    T: WithBaseField + GodotClass<Base = Node3D>,
{
    match Gd::try_from_instance_id(instance_id) {
        Ok(node) => Ok(node),
        Err(e) => Err(format!("node_from_instance_id: {}", e.to_string())),
    }
}

pub trait HasCUID2Field {
    fn get_cuid2(&self) -> String;
    fn set_cuid2(&mut self, cuid2: String);
}

pub trait HasHandleField {
    fn get_handle(&self) -> Handle;
    fn set_handle(&mut self, handle: Handle);
}
