@tool
extends Node3D

const project_settings = preload("res://addons/godot-rapier-3d/gdscript/project_settings.gd")
const debug_lines = preload("res://addons/godot-rapier-3d/gdscript/debug_lines.gd")
const physics_controls = preload("res://addons/godot-rapier-3d/gdscript/physics_controls.tscn")

# Config
@onready var run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
@onready var run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
@onready var show_debug_outlines = ProjectSettings.get_setting("debug/rapier_3d/show_debug_outlines")
@onready var show_ui = ProjectSettings.get_setting("debug/rapier_3d/show_ui")

signal settings_changed

var debug_render_pipeline
var debug_lines_node
var debug_ui_node

func _enter_tree():
	var project_settings_node = project_settings.new()
	project_settings_node.add_project_settings()
	project_settings_node.free()
	ProjectSettings.connect("settings_changed", self._on_settings_changed)
	update_log_level()

func _exit_tree():
	ProjectSettings.disconnect("settings_changed", self._on_settings_changed)
	if debug_lines_node != null: debug_lines_node.free()
	if debug_ui_node != null: debug_ui_node.free()
	debug_render_pipeline = null

func _on_settings_changed():
	run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
	run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
	show_debug_outlines = ProjectSettings.get_setting("debug/rapier_3d/show_debug_outlines")
	show_ui = ProjectSettings.get_setting("debug/rapier_3d/show_ui")
	if debug_lines_node != null: debug_lines_node.clear_lines()
	update_log_level()
	settings_changed.emit()

func update_log_level():
	if is_instance_valid(Rapier3DEngine): Rapier3DEngine.set_log_level(ProjectSettings.get_setting("debug/rapier_3d/logging_level"))

func _spawn_ui():
	debug_ui_node = physics_controls.instantiate()
	if Engine.is_editor_hint():
		#EditorInterface.get_editor_viewport_3d().add_child(debug_ui_node) # TODO - appears but buttons are not clickable - not sure how to fix
		pass
	else:
		add_child(debug_ui_node)

func _ready():
	if !_should_run(): return
	debug_lines_node = debug_lines.new()
	add_child(debug_lines_node)
	
	_spawn_ui()
	
	_create_debugger()
	GDExtensionManager.connect("extensions_reloaded", _create_debugger)

func _create_debugger():
	debug_render_pipeline = RapierDebugRenderPipeline.new()
	debug_render_pipeline.register_debugger(self)

func _process(_delta):
	if !_should_run(): return
	if show_debug_outlines:
		debug_render_pipeline.render()

func _draw_line(a, b, color):
	if debug_lines_node: debug_lines_node.draw_line(a, b, color)

func _should_run():
	if Engine.is_editor_hint(): return run_in_editor
	else: 
		var current_scene = get_tree().current_scene
		var scene_path = current_scene.scene_file_path
		var is_test_scene = scene_path.contains("res://tests/")
		if is_test_scene and !OS.is_debug_build(): return false
		return run_in_game
