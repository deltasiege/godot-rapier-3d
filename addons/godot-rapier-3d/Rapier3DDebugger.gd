@tool
extends Node3D

var show_colliders = true
var debug_render_pipeline

func _ready():
	print("Debugger _ready")
	debug_render_pipeline = RapierDebugRenderPipeline.new()
	debug_render_pipeline.register_debugger(self)

func _process(_delta):
	if show_colliders: render_colliders()

func render_colliders():
	pass
	#debug_render_pipeline.render_colliders()

func _draw_line(a, b, color):
	print("Godot drawing line: ", a, b, color)
