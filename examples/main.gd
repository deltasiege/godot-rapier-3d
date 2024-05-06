extends Node3D

func _ready():
	Rapier3DEngine.print_debug_info()

func _physics_process(_delta):
	Rapier3D.step()
