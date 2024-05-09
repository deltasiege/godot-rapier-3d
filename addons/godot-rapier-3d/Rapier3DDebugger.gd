@tool
extends Node3D

var show_colliders = true
var debug_render_pipeline

const debug_lines_res = preload("res://addons/godot-rapier-3d/src/gdscript/debug_lines.gd")
var _debug_lines

#func _ready():
	#print("Debugger _ready")
	#
	#_debug_lines = debug_lines_res.new()
	#add_child(_debug_lines)
	#
	#debug_render_pipeline = RapierDebugRenderPipeline.new()
	#debug_render_pipeline.register_debugger(self)
#
#func _process(_delta):
	#if show_colliders: render_colliders()
#
#func render_colliders():
	#debug_render_pipeline.render_colliders()
#
#func _draw_line(a, b, color):
	#if _debug_lines: _debug_lines.draw_line(a, b, color)
	#pass
