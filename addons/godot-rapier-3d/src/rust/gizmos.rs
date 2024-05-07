use crate::editor_plugin::GodotRapier3DEditorPlugin;
use godot::engine::EditorNode3DGizmoPlugin;
use godot::engine::GDScript;
use godot::prelude::*;

pub const GIZMO_PATHS: &'static [&'static str] =
    &["res://addons/godot-rapier-3d/src/gdscript/gizmos/collider3D.gd"];

pub fn add_all_gizmos(plugin: &mut GodotRapier3DEditorPlugin) {
    for path in GIZMO_PATHS {
        add_gizmo(plugin, path);
    }
}

pub fn remove_all_gizmos(plugin: &mut GodotRapier3DEditorPlugin) {
    let ston = crate::utils::get_engine_singleton();
    if ston.is_some() {
        let mut singleton = ston.unwrap();
        let gizmo_iids = &mut singleton.bind_mut().gizmo_iids;
        for iid in gizmo_iids.clone() {
            remove_gizmo(plugin, iid);
        }
        gizmo_iids.clear();
    }
}

fn add_gizmo(plugin: &mut GodotRapier3DEditorPlugin, path: &str) {
    if let Ok(mut gizmo_script) = try_load::<GDScript>(path) {
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let gizmo_iids = &mut singleton.bind_mut().gizmo_iids;

            let script_obj: Variant = gizmo_script.instantiate(&[]);
            let casted = Gd::<EditorNode3DGizmoPlugin>::from_variant(&script_obj);
            let iid = casted.instance_id().to_i64();
            plugin.base_mut().add_node_3d_gizmo_plugin(casted);
            gizmo_iids.push(iid);
            godot_print!("Added gizmo: {:?}", path);
        }
    } else {
        godot_error!("Could not load gizmo: {:?}", path);
    }
}

fn remove_gizmo(plugin: &mut GodotRapier3DEditorPlugin, iid: i64) {
    let instance_id = InstanceId::from_i64(iid);
    let gizmo: Gd<EditorNode3DGizmoPlugin> = match Gd::try_from_instance_id(instance_id) {
        Ok(gizmo) => gizmo,
        _ => {
            godot_error!("Could not remove gizmo {:?}", instance_id);
            return;
        }
    };
    plugin.base_mut().remove_node_3d_gizmo_plugin(gizmo);
}
