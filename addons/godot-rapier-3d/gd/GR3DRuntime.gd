@tool
extends Node

## Godot Rapier 3D game runtime functionality goes here

@export var autoplay = true
var playing: bool = false
var paused: bool = false

var pending_actions = []

@onready var DrawLine = preload("./draw_line.gd").new()

func _ready():
	if Engine.is_editor_hint(): return
	add_child(DrawLine)
	if autoplay: play()

func _exit_tree():
	DrawLine.queue_free()

func _process(_delta):
	if Engine.is_editor_hint(): return
	if get_tree().debug_collisions_hint: DrawLine.draw_lines(GR3D._get_debug_lines())

func _physics_process(_delta):
	if Engine.is_editor_hint(): return
	if playing and !paused: GR3D.step(1)

func play():
	playing = true
	paused = false

func pause():
	playing = false
	paused = true

func toggle_pause(_paused: bool):
	playing = !_paused
	paused = _paused

func _draw_line(origin: Vector3, end: Vector3):
	DrawLine.draw_line(origin, end, Color.WHITE)
