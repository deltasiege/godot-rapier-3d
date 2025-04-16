extends RollbackKinematicCharacter3D

var _cam: Camera3D

func on_network_spawn(_data: Dictionary):
	# Hmm cant do anything in here that needs to be done during rollbacks - is that a problem?
	_cam = self.find_children("*", "Camera3D", true, false)[0]
	if is_multiplayer_authority(): _cam.make_current()

func _physics_process(_delta):
	movement()

func movement():
	if !_cam: return
	var input_dir = GR3D.get_input("move", self)
	var relative_dir = (_cam.get_parent().transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	var move_amt = relative_dir
	move_by_amount(move_amt)
