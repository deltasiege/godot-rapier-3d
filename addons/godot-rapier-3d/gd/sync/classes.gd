class ActionBufferStep:
	var tick: int
	var num_actions: int
	var hash: int
	var serialized: PackedByteArray
	
	func to_arr() -> Array:
		return [tick, num_actions, hash, serialized]
	
	func from_arr(arr: Array) -> void:
		if arr.size() < 4: push_error("Trying to deserialize invalid ActionBufferStep: ", arr)
		tick = arr[0]
		num_actions = arr[1]
		hash = arr[2]
		serialized = arr[3]
		
