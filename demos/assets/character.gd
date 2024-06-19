extends RapierCharacterBody3D

@export var speed = 0.1
@export var deceleration = 5.0
var velocity = Vector3.ZERO

func _physics_process(_delta):
	var input = Input.get_vector("left", "right", "forward", "back")
	var dir = Vector3(input.x, 0, input.y)
	if dir != Vector3.ZERO: velocity = dir * speed
	else: velocity = velocity.move_toward(Vector3.ZERO, deceleration)
	move_character(velocity, 1)
