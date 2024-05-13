const utils = preload("res://addons/godot-rapier-3d/gdscript/utils.gd")

static func _get_physics_state(root: Node3D):
	var state = PackedByteArray()
	var physics_objects = _get_all_physics_objects(root)
	var sorted = _sort_by_iid(physics_objects)
	for obj: Node3D in physics_objects:
		var pos = obj.global_transform.origin
		var bsis = obj.global_transform.basis
		state.append_array(var_to_bytes(pos))
		state.append_array(var_to_bytes(bsis))
	return state

static func _get_all_physics_objects(root: Node3D) -> Array[Node3D]:
	var physics_objects: Array[Node3D] = []
	utils._append_children_by_class("RapierRigidBody3D", root, physics_objects)
	utils._append_children_by_class("RapierCollider3D", root, physics_objects)
	return physics_objects

static func _sort_by_iid(nodes: Array[Node3D]):
	var arr = nodes.duplicate(true)
	arr.sort_custom(_compare_iid)
	return arr

#Returns true if B has greater instance id than A
static func _compare_iid(a, b):
	if a.get_instance_id() < b.get_instance_id(): return true
	return false

static func _check_sort(nodes: Array[Node3D]):
	for node in nodes: print("IID: ", node.get_instance_id())
