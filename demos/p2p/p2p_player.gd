extends Node

@export var peer_id: int = 1 :
	set(id):
		peer_id = id
		set_multiplayer_authority(id, false)

@export var controller: Node3D
@export var spawn_pos: Vector3
@export var cam_pivot: Node3D

func _on_enter_tree():
	$MultiplayerSynchronizer.set_multiplayer_authority(peer_id)

func _ready():
	var cam1 = find_child("*Camera*", true, false)
	var cam2 = find_child("*cam*", true, false)
	var cam = cam1 if cam1 != null else cam2
	cam.current = peer_id == multiplayer.get_unique_id()
	
	if controller: 
		if controller.has_method("set_uid"): controller.set_uid(GR3D.create_cuid())
		controller.global_position = spawn_pos

func _get_local_input() -> Dictionary:
	var dir = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	var direction = (cam_pivot.transform.basis * Vector3(dir.x, 0, dir.y)).normalized()
	var jump_pressed = Input.is_action_just_pressed("jump")
	
	var input := {}
	if direction != Vector3.ZERO:
		input['direction'] = direction
	input['jump_pressed'] = jump_pressed
	return input

func _network_process(_input: Dictionary):
	pass

func _save_state() -> Dictionary:
	return {}

func _load_state(_state: Dictionary):
	pass
