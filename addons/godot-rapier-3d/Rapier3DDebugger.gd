@tool
extends Node3D

const project_settings = preload("res://addons/godot-rapier-3d/gdscript/project_settings.gd")

# Config
@onready var run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
@onready var run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
@onready var show_colliders = ProjectSettings.get_setting("debug/rapier_3d/show_colliders")

var debug_render_pipeline

const debug_lines_res = preload("res://addons/godot-rapier-3d/gdscript/debug_lines.gd")
var _debug_lines

func _enter_tree():
	var project_settings_node = project_settings.new()
	project_settings_node.add_project_settings()
	project_settings_node.free()
	ProjectSettings.connect("settings_changed", self._on_settings_changed)

func _exit_tree():
	ProjectSettings.disconnect("settings_changed", self._on_settings_changed)

func _on_settings_changed():
	run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
	run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
	show_colliders = ProjectSettings.get_setting("debug/rapier_3d/show_colliders")
	if _debug_lines != null: _debug_lines.clear_lines()
	if is_instance_valid(Rapier3DEngine): Rapier3DEngine.set_log_level(ProjectSettings.get_setting("debug/rapier_3d/logging_level"))

func _ready():
	if !_should_run(): return
	_debug_lines = debug_lines_res.new()
	add_child(_debug_lines)
	
	debug_render_pipeline = RapierDebugRenderPipeline.new()
	debug_render_pipeline.register_debugger(self)

func _process(_delta):
	if !_should_run(): return
	if show_colliders: _render_colliders()

func _render_colliders():
	debug_render_pipeline.render_colliders()

func _draw_line(a, b, color):
	if _debug_lines: _debug_lines.draw_line(a, b, color)
	pass

func _should_run():
	if Engine.is_editor_hint(): return run_in_editor
	else: return run_in_game
