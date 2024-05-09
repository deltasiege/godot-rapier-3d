extends Node3D

func _physics_process(_delta):
	Rapier3D.step()
