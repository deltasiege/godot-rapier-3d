extends Object

var _property_order: int = 1000

var _project_settings = [
	{ "name": "debug/rapier_3d/logging_level", "type": TYPE_STRING, "default": "Info", "hint": PROPERTY_HINT_ENUM, "hint_string": "Off,Error,Warning,Info,Debug,Trace" },
	{ "name": "debug/rapier_3d/debug_in_game", "type": TYPE_BOOL, "default": true },
	{ "name": "debug/rapier_3d/debug_in_editor", "type": TYPE_BOOL, "default": true },
	{ "name": "debug/rapier_3d/show_colliders", "type": TYPE_BOOL, "default": true },
	{ "name": "debug/rapier_3d/show_ui", "type": TYPE_BOOL, "default": true },
	{ "name": "physics/rapier_3d/gravity_vector", "type": TYPE_VECTOR3, "default": Vector3(0, -9.8, 0) },
]

func _add_project_setting(name: String, type: int, default, hint = null, hint_string = null, restart_if_changed = false) -> void:
	if not ProjectSettings.has_setting(name): ProjectSettings.set_setting(name, default)
	ProjectSettings.set_initial_value(name, default)
	ProjectSettings.set_order(name, _property_order)
	_property_order += 1
	var info := { name = name, type = type, }
	if hint != null: info['hint'] = hint
	if hint_string != null: info['hint_string'] = hint_string
	ProjectSettings.add_property_info(info)
	ProjectSettings.set_restart_if_changed(name, restart_if_changed)

func _remove_project_setting(name: String) -> void:
	if ProjectSettings.has_setting(name): ProjectSettings.set_setting(name, null)

func add_project_settings() -> void:
	for setting in _project_settings:
		_add_project_setting(setting.name, setting.type, setting.default, setting.get("hint", null), setting.get("hint_string", null), setting.get("restart_if_changed", false))

func remove_project_settings() -> void:
	for setting in _project_settings:
		_remove_project_setting(setting.name)
