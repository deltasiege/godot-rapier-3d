@tool
extends Node3D

const utils = preload("res://addons/godot-rapier-3d/gdscript/utils.gd")
const physics_state = preload("res://addons/godot-rapier-3d/gdscript/physics_state.gd")

func _ready():
	Rapier3DEngine._process()

func _physics_process(_delta):
	Rapier3DEngine._process()

func step() -> void:
	Rapier3DEngine.step()

func get_state() -> PackedByteArray:
	return Rapier3DEngine.get_state()

func set_state(physics_state: PackedByteArray):
	Rapier3DEngine.set_state(physics_state)

func get_hash(physics_state: PackedByteArray) -> int:
	return Array(physics_state.compress()).hash()

func get_godot_state() -> PackedByteArray:
	var ret = physics_state._get_physics_state(get_tree().current_scene)
	return ret

func get_godot_hash(physics_state: PackedByteArray) -> int:
	return Array(physics_state.compress()).hash()
