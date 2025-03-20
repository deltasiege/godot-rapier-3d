@tool
extends EditorScript

## Iterates over all GR3D objects and re-generates their
## CUIDs if missing or duplicated

## You may need to open + close sub-scenes after running as well
## if you still get errors when playing your game

## A CUID is a unique ID that has a low collision chance. 
## You can read more about them here: https://github.com/paralleldrive/cuid

const LOG_PREFIX = "[GR3D][fixup_cuids]: "

func _run():
	msg("Starting")
	var dupes = fixup_cuids(get_scene())
	if dupes > 0: msg("Completed. " + str(dupes) + " duplicate CUIDs still remain. Please run this script again")
	else: msg("Completed successfully. " + str(dupes) + " duplicates remain.")

## Regenerates any missing or clashing CUIDS and then returns the number of
## remaining duplicates
static func fixup_cuids(root: Node, max_iters: int = 10) -> int:
	var objs = get_all_objects(root)
	msg("Checking " + str(objs.size()) + " GR3D nodes in current scene")
	var cuids = []
	for node in objs:
		var cuid = node.get_meta("cuid", false)
		if !cuid:
			cuid = new_cuid(node)
			print("'" + node.name + "' is missing CUID. Got new: " + cuid)
		var iters = 0
		while cuids.has(cuid):
			var clashing = cuid
			cuid = new_cuid(node)
			print("CUID '" + clashing + "' already exists. Got new: " + cuid)
			iters += 1
			if iters > max_iters:
				push_error("[GR3D][fixup_cuids]: Couldn't get a unique cuid after " + str(max_iters) + " attempts.")
				break
		msg("OK: " + node.name + " : " + cuid)
		cuids.push_back(cuid)
	
	var duplicates = 0
	for cuid in cuids:
		var instances = cuids.filter(func(what): return what == cuid)
		if instances.size() > 1: duplicates += 1
	return duplicates

static func get_all_objects(root: Node) -> Array[Node3D]:
	var areas = root.find_children("*", "RapierArea3D", true, false)
	var col_shapes = root.find_children("*", "RapierCollisionShape3D", true, false)
	var kinematic_chars = root.find_children("*", "RapierKinematicCharacter3D", true, false)
	var pid_chars = root.find_children("*", "RapierPIDCharacter3D", true, false)
	var rbs = root.find_children("*", "RapierRigidBody3D", true, false)
	var static_bodies = root.find_children("*", "RapierStaticBody3D", true, false)
	var all: Array[Node3D] = []
	all.append_array(areas)
	all.append_array(col_shapes)
	all.append_array(kinematic_chars)
	all.append_array(rbs)
	all.append_array(static_bodies)
	return all

static func new_cuid(node: Node3D) -> String:
	var cuid = GR3D.create_cuid()
	node.set_meta("cuid", cuid)
	return cuid

static func msg(msg: String):
	print(LOG_PREFIX + msg)
