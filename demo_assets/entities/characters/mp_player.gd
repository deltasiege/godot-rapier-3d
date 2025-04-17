extends RollbackKinematicCharacter3D

var _cam: Camera3D
var _cam_forward

func on_network_spawn(_data: Dictionary):
	# Hmm cant do anything in here that needs to be done during rollbacks - is that a problem?
	_cam = self.find_children("*", "Camera3D", true, false)[0]
	if is_multiplayer_authority(): _cam.make_current()


func _physics_process(_delta):
	if !_cam: return
	var cam_dir = _cam.get_parent().transform.basis
	#GR3D.set_state(gruid, cam_dir) # get gruid from self node is ok

static func on_physics_tick(gruid: String, node_state: Dictionary):
	print("DOING! ")
	#var input_dir = GR3D.get_input(gruid, "move")
	#var relative_dir = (node_state.cam_dir * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	#
	#var move_amt = relative_dir
	#GR3D.move_by_amount(gruid, move_amt)
