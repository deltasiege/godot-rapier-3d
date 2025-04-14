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

// Returns the rollback node class of the resource at the given path.
// TODO delme
// pub fn get_rb_resource_class_name(resource_path: &String) -> Option<RollbackNodeClass> {
//     let scene = match load_scene(resource_path) {
//         Some(scene) => scene,
//         None => {
//             log::error!("Failed to load scene at path: {}", resource_path);
//             return None;
//         }
//     };

//     let state = scene.get_state();
//     let class = match RollbackNodeClass::try_from() {
//         Ok(class) => class,
//         Err(_) => {
//             log::error!(
//                 "Invalid rollback node class '{}' found at resource path: {}",
//                 scene.get_class(),
//                 resource_path
//             );
//             return None;
//         }
//     };

//     Some(class)
// }
