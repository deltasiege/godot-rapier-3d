extends RollbackKinematicCharacter3D

var _cam: Camera3D

# Occurs at the end of rollbacks when the Godot node is created
func on_network_spawn(_data: Dictionary):
	_cam = self.find_children("*", "Camera3D", true, false)[0]
	if is_multiplayer_authority(): _cam.make_current()

func _physics_process(_delta):
	if !_cam: return
	var cam_dir = _cam.get_parent().transform.basis
	GR3D.set_state(get_gruid(), "cam_dir", cam_dir)

static func get_local_input(node_state: Dictionary):
	pass

static func on_physics_tick(peer_id: int, gruid: String):
	var cam_relative_dir = GR3D.get_input(gruid, "move")
	var move_amt = cam_relative_dir
	if move_amt != Vector3.ZERO: print(peer_id, " is moving player " + gruid + " by: ", move_amt)
	GR3D.move_by_amount(gruid, move_amt)
