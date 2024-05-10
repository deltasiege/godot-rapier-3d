use crate::editor_plugin::GR3DEditorPlugin;
use crate::log::LogLevel;
use godot::engine::EditorNode3DGizmoPlugin;
use godot::engine::GDScript;
use godot::prelude::*;

pub const GIZMO_PATHS: &'static [&'static str] = &[
    "res://addons/godot-rapier-3d/gdscript/gizmos/collider3D.gd",
];

pub fn add_all_gizmos(plugin: &mut GR3DEditorPlugin) {
    for path in GIZMO_PATHS {
        add_gizmo(plugin, path);
    }
}

pub fn remove_all_gizmos(plugin: &mut GR3DEditorPlugin) {
    let mut engine = crate::get_engine!();
    let mut bind = engine.bind_mut();
    for iid in bind.gizmo_iids.clone() {
        remove_gizmo(plugin, iid, bind.log_level);
    }
    bind.gizmo_iids.clear();
}

fn add_gizmo(plugin: &mut GR3DEditorPlugin, path: &str) {
    let mut engine = crate::get_engine!();
    let mut bind = engine.bind_mut();
    let mut gizmo_script = match try_load::<GDScript>(path) {
        Ok(script) => script,
        _ => {
            crate::error!(bind; "Could not load gizmo: {:?}", path);
            return;
        }
    };

    let script_obj: Variant = gizmo_script.instantiate(&[]);
    let gizmo = match Gd::<EditorNode3DGizmoPlugin>::try_from_variant(&script_obj) {
        Ok(gizmo) => gizmo,
        Err(error) => {
            crate::error!(bind; "Could not process gizmo script {:?}. Error: {:?}", path, error);
            return;
        }
    };
    let iid = gizmo.instance_id().to_i64();
    plugin.base_mut().add_node_3d_gizmo_plugin(gizmo);
    bind.gizmo_iids.push(iid);
    crate::debug!(bind; "Added gizmo: {:?}", path);
}

fn remove_gizmo(plugin: &mut GR3DEditorPlugin, iid: i64, log_level: LogLevel) {
    let instance_id = InstanceId::from_i64(iid);
    let gizmo: Gd<EditorNode3DGizmoPlugin> = match Gd::try_from_instance_id(instance_id) {
        Ok(gizmo) => gizmo,
        _ => {
            crate::error!(log_level => "Could not remove gizmo {:?}", instance_id);
            return;
        }
    };
    plugin.base_mut().remove_node_3d_gizmo_plugin(gizmo);
}
