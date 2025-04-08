@tool
extends EditorScript

## Iterates over all GR3D objects and re-generates their
## GRUIDs if missing or duplicated

## You may need to open + close sub-scenes after running
## if you still get errors when playing your game

## A GRUID is a Godot Rollback unique identifier that
## identifies a rollback node across all network peers

const LOG_PREFIX = "[GR3D][fixup_gruids]: "

func _run():
	msg("Starting")
	set_ambient_gruids(get_scene())
	msg("Completed successfully")

## Resets gruids for every rollback node in the current scene
## using a peer_index of 0 (ambient)
static func set_ambient_gruids(root: Node):
	var objs = get_all_objects(root)
	msg("Resetting GRUIDs for " + str(objs.size()) + " GR3D nodes in current scene")
	for idx in objs.size():
		var node = objs[idx]
		var gruid = [0, idx]
		node.set_meta("gruid", gruid)
		msg("OK: " + node.name + " : " + str(gruid))

static func get_all_objects(root: Node) -> Array[Node3D]:
	var areas = root.find_children("*", "RollbackArea3D", true, false)
	var col_shapes = root.find_children("*", "RollbackCollisionShape3D", true, false)
	var kinematic_chars = root.find_children("*", "RollbackKinematicCharacter3D", true, false)
	var pid_chars = root.find_children("*", "RollbackPIDCharacter3D", true, false)
	var rbs = root.find_children("*", "RollbackRigidBody3D", true, false)
	var static_bodies = root.find_children("*", "RollbackStaticBody3D", true, false)
	var all: Array[Node3D] = []
	all.append_array(areas)
	all.append_array(col_shapes)
	all.append_array(kinematic_chars)
	all.append_array(rbs)
	all.append_array(static_bodies)
	return all

static func msg(msg: String):
	print(LOG_PREFIX + msg)
