@tool
extends EditorPlugin

func _enter_tree():
	add_autoload_singleton("Rapier3D", "res://addons/godot-rapier-3d/Rapier3D.gd")

func _exit_tree():
	pass
