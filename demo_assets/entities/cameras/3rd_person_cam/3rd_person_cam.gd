extends SpringArm3D

@export var mouse_sensitivity: float = 0.005

func _process(delta):
	var vel = Input.get_last_mouse_velocity() * mouse_sensitivity
	rotation.y -= vel.x
	rotation.y = wrapf(rotation.y, 0.0, TAU)
	
	rotation.x -= vel.y
	rotation.x = clamp(rotation.x, -PI/2, PI/4)

#func _unhandled_input(event: InputEvent) -> void:
	#if event is InputEventMouseMotion:
		#rotation.y -= event.relative.x * mouse_sensitivity
		#rotation.y = wrapf(rotation.y, 0.0, TAU)
		#
		#rotation.x -= event.relative.y * mouse_sensitivity
		#rotation.x = clamp(rotation.x, -PI/2, PI/4)
