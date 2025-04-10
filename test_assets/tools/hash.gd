static var FixupGRUIDS = preload("res://addons/godot-rollback-3d/gd/fixup_gruids.gd")

static func get_gr3d_hash() -> int:
	var state: PackedByteArray = GR3D.save_snapshot()
	return Array(state.compress()).hash()

static func get_godot_hash(root: Node) -> int:
	var state = _get_physics_state(root)
	return Array(state.compress()).hash()

static func _get_physics_state(root: Node) -> PackedByteArray:
	var state = PackedByteArray()
	var physics_objects = FixupGRUIDS.get_all_objects(root)
	var sorted = _sort_by_iid(physics_objects)
	for obj: Node3D in sorted: state.append_array(var_to_bytes(obj.global_transform))
	return state

static func _sort_by_iid(nodes: Array[Node3D]):
	var arr = nodes.duplicate(true)
	arr.sort_custom(_compare_iid)
	return arr

# Returns true if B has greater instance id than A
static func _compare_iid(a, b):
	if a.get_instance_id() < b.get_instance_id(): return true
	return false
