extends RollbackKinematicCharacter3D

func on_network_spawn(_data: Dictionary):
	# Hmm cant do anything in here that needs to be done during rollbacks - is that a problem?
	if is_multiplayer_authority():
		var cam: Camera3D = self.find_children("*", "Camera3D", true, false)[0]
		cam.make_current()

func _physics_process(_delta):
	move_by_amount(Vector3.ONE)
