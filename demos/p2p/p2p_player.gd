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
