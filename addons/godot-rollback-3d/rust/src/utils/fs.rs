use godot::prelude::*;

pub fn load_scene(scene_path: &String) -> Option<Gd<PackedScene>> {
    load_path::<PackedScene>(scene_path)
}

pub fn load_path<T: Inherits<Resource>>(resource_path: &String) -> Option<Gd<T>> {
    match try_load(resource_path) {
        Ok(loaded) => Some(loaded),
        Err(err) => {
            log::error!(
                "Failed to load resource from path '{}': {}",
                resource_path,
                err
            );
            None
        }
    }
}
