@tool
extends Node

## Godot Rapier 3D game runtime functionality goes here

@export var autoplay = true
var playing: bool = false
var paused: bool = false

var pending_actions = []

@onready var DrawLine = preload("./draw_line.gd").new()
@onready var PingTimer = preload("./ping_timer.gd").new()
const DEFAULT_NETWORK_ADAPTER_PATH = "res://addons/godot-rapier-3d/gd/rpc_adapter.gd"

func _ready():
	if Engine.is_editor_hint(): return
	_add_child_modules()
	if autoplay: play()

func _exit_tree():
	DrawLine.queue_free()

func _process(_delta):
	if Engine.is_editor_hint(): return
	if get_tree().debug_collisions_hint: DrawLine.draw_lines(GR3D._get_debug_lines())

func _physics_process(_delta):
	if Engine.is_editor_hint(): return
	if playing and !paused: GR3D.step(1)
	GR3DNet.on_physics_process()

func play():
	playing = true
	paused = false

func pause():
	playing = false
	paused = true

func toggle_pause(_paused: bool):
	playing = !_paused
	paused = _paused

func _add_child_modules():
	add_child(DrawLine)
	add_child(PingTimer)
	_add_network_adapter()

func _add_network_adapter():
	var network_adapter = load(DEFAULT_NETWORK_ADAPTER_PATH).new()
	add_child(network_adapter)
	GR3DNet._attach_network_adapter(network_adapter)

func _draw_line(origin: Vector3, end: Vector3):
	DrawLine.draw_line(origin, end, Color.WHITE)
