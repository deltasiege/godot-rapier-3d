static var FixupCUIDS = preload("res://addons/godot-rapier-3d/gd/fixup_cuids.gd")

static func get_rapier_hash() -> int:
	var state: PackedByteArray = GR3D.save_snapshot()
	return Array(state.compress()).hash()

static func get_godot_hash(root: Node) -> int:
	var state = _get_physics_state(root)
	return Array(state.compress()).hash()

static func _get_physics_state(root: Node) -> PackedByteArray:
	var state = PackedByteArray()
	var physics_objects = FixupCUIDS.get_all_objects(root)
	var sorted = _sort_by_iid(physics_objects)
	for obj: Node3D in sorted:
		var pos = obj.global_transform.origin
		var bsis = obj.global_transform.basis
		state.append_array(var_to_bytes(pos))
		state.append_array(var_to_bytes(bsis))
	return state

static func _sort_by_iid(nodes: Array[Node3D]):
	var arr = nodes.duplicate(true)
	arr.sort_custom(_compare_iid)
	return arr

#Returns true if B has greater instance id than A
static func _compare_iid(a, b):
	if a.get_instance_id() < b.get_instance_id(): return true
	return false
