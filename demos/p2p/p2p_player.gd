extends Node

# Set by the authority, synchronized on spawn.
@export var peer_id: int = 1 :
	set(id):
		peer_id = id
		# Give authority over the player input to the appropriate peer.
		if has_node("InputSynchronizer"): $InputSynchronizer.set_multiplayer_authority(id)

@export var controller: Node3D
@export var spawn_pos: Vector3

func _ready():
	var cam1 = find_child("*Camera*", true, false)
	var cam2 = find_child("*cam*", true, false)
	var cam = cam1 if cam1 != null else cam2
	cam.current = peer_id == multiplayer.get_unique_id()
	
	if controller: 
		if controller.has_method("set_uid"): controller.set_uid(GR3D.create_cuid())
		controller.global_position = spawn_pos
