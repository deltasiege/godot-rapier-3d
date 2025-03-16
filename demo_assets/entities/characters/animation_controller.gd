extends Node

@export var character: Node3D
@export var state_machine: Node3D

func _physics_process(_delta):
	var char_vel = character.get_real_velocity()
	if not character.is_on_floor:
		if char_vel.y < 0: state_machine.fall()
		else: state_machine.jump()
	elif char_vel.length() > 1:
		state_machine.move()
	else:
		state_machine.idle()
