static func get_or_create_ab_frame(sync: GR3DSync, tick: int, action_buffer: Array) -> GR3DActionFrame:
	var ab_frame: GR3DActionFrame
	if action_buffer.size() == 0:
		ab_frame = GR3DActionFrame.new(tick)
		action_buffer.append(ab_frame)
	elif tick > action_buffer[-1].tick:
		var highest = action_buffer[-1].tick
		while highest < tick:
			highest += 1
			ab_frame = GR3DActionFrame.new(highest)
			action_buffer.append(ab_frame)
	else:
		ab_frame = get_ab_frame(sync, tick)
		if ab_frame == null:
			return sync.handle_fatal_error("Requested input frame (%s) not found in buffer" % tick)
	return ab_frame

static func get_ab_frame(sync: GR3DSync, tick: int) -> GR3DActionFrame:
	if tick < sync.action_buffer_start_tick:
		return null
	var index = tick - sync.action_buffer_start_tick
	if index >= sync.action_buffer.size():
		return null
	var ab_frame = sync.action_buffer[index]
	assert(ab_frame.tick == tick, "Input frame retreived from input buffer has mismatched tick number")
	return ab_frame
