extends Node3D

#const debug = preload("res://addons/godot-rapier-3d/src/gdscript/debug.gd")
#const rb = preload("res://addons/godot-rapier-3d/src/gdscript/rigidbody.gd")
#const utils = preload("res://addons/godot-rapier-3d/src/gdscript/utils.gd")

static func step():
	Rapier3DEngine.step()
