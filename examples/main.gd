extends Node3D

func _ready():
	Rapier3D.print_debug_info()

func _physics_process(_delta):
	Rapier3D.step()
