extends Node3D

signal physics_ready

func _physics_process(_delta):
	if Engine.get_physics_frames() != 1: return # Need to wait a few frames for colliders to properly mount in the tree
	physics_ready.emit()

func step() -> void:
	Rapier3DEngine.step()

func get_state() -> PackedByteArray:
	return Rapier3DEngine.get_state()

func set_state(physics_state: PackedByteArray):
	Rapier3DEngine.set_state(physics_state)

func get_hash(physics_state: PackedByteArray) -> int:
	return Array(physics_state.compress()).hash()
