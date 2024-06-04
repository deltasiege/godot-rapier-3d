extends RapierCharacterBody3D

func _physics_process(delta):
	if Engine.get_physics_frames() % 5 != 0: return
	move_and_slide(Vector3(1, 0, 1) * 0.1, delta)
