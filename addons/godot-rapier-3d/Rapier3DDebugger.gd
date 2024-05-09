@tool
extends Node3D

# Config
var run_in_editor = true
var run_in_game = false
var show_colliders = true

var debug_render_pipeline

const debug_lines_res = preload("res://addons/godot-rapier-3d/gdscript/debug_lines.gd")
var _debug_lines

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
