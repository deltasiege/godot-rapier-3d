## This queue tries to ensure that actions requested of Rapier on each frame
## occur in the same order across different machines

enum ACTION_TYPE {
	ADD_NODE,
	REMOVE_NODE,
	MOVE_NODE,
	CONFIGURE_NODE,
	STEP,
}

static func add_action(queue: Array, type: ACTION_TYPE, data: Dictionary = {}):
	match type:
		ACTION_TYPE.ADD_NODE, ACTION_TYPE.REMOVE_NODE:
			var node = data.node
			if !is_instance_valid(node): return
			queue.append({ "cuid": get_cuid(node), "type": type, "node": node })
		ACTION_TYPE.MOVE_NODE:
			var node = data.node
			if !is_instance_valid(node): return
			queue.append({ "cuid": get_cuid(node), "type": type, "node": node, "desired_movement": data.desired_movement })
		ACTION_TYPE.CONFIGURE_NODE:
			var node = data.node
			if !is_instance_valid(node): return
			queue.append({ "cuid": get_cuid(node), "type": type, "node": node })
		ACTION_TYPE.STEP:
			queue.append({ "type": type, "count": data.get("count", 1) })

static func _sort_queue(queue: Array):
	queue.sort_custom(func(a: Dictionary, b: Dictionary):
		var uid = a.get("cuid", "").naturalnocasecmp_to(b.get("cuid", "")) if (a.get("cuid") != null and b.get("cuid") != null) else 0
		var type = a.type - b.type
		var mag = a.get("magnitude", 0) - b.get("magnitude", 0)
		if b.type == ACTION_TYPE.STEP and a.type != ACTION_TYPE.STEP: return true # Sort STEPS after everything else
		elif a.type == ACTION_TYPE.STEP and b.type != ACTION_TYPE.STEP: return false
		elif uid != 0: return uid < 0
		elif type != 0: return type
		elif mag != 0: return mag
		else: return false
	)

static func process_queue(queue: Array):
	_sort_queue(queue)
	for action in queue:
		match action.type:
			ACTION_TYPE.ADD_NODE:
				GR3D._add_nodes([action.node])
			ACTION_TYPE.REMOVE_NODE:
				GR3D._remove_nodes([action.node])
			ACTION_TYPE.MOVE_NODE:
				GR3D._move_nodes([action.node], action.desired_movement)
			ACTION_TYPE.CONFIGURE_NODE:
				GR3D._configure_nodes([action.node])
			ACTION_TYPE.STEP:
				GR3D.step(action.count)
	queue.clear()

static func get_cuid(node: Node3D) -> String:
	var cuid = node.get_meta("cuid", false)
	if !cuid: 
		push_error("[GR3D][queue]: Could not retrieve cuid of '" + node.name + "'. Please run 'addons/godot-rapier-3d/gd/fixup_cuids.gd' to resolve.")
		return ""
	return cuid
