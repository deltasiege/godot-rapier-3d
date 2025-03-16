@tool
extends Node

## Godot Rapier 3D game runtime functionality goes here

@export var autoplay = true
var playing: bool = false
var paused: bool = false

var pending_actions = []

var Queue = preload("./queue.gd")
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
	if playing and !paused:
		Queue.add_action(pending_actions, Queue.ACTION_TYPE.STEP)
		Queue.process_queue(pending_actions)

func _add_node(node): Queue.add_action(pending_actions, Queue.ACTION_TYPE.ADD_NODE, { "node": node })
func _configure_node(node): Queue.add_action(pending_actions, Queue.ACTION_TYPE.CONFIGURE_NODE, { "node": node })
func _remove_node(node): Queue.add_action(pending_actions, Queue.ACTION_TYPE.REMOVE_NODE, { "node": node })
func _move_node(node, desired_movement: Vector3): Queue.add_action(pending_actions, Queue.ACTION_TYPE.MOVE_NODE, { "node": node, "desired_movement": desired_movement })

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
