class_name ControlsMgr

# Adds a set of actions to the input map
# Usage:
#	actions = [{ "name": "action_name", "type": InputEventMouseButton, "btn_idx": MOUSE_BUTTON_WHEEL_UP, "<OPTIONAL MODIFIERS> (shift | alt | ctrl | cmd | meta)": true }]
#	add_input_map_actions(actions)
static func add_input_map_actions(actions: Array):
	for a in actions:
		if InputMap.has_action(a.name): continue
		InputMap.add_action(a.name)
		var ev = a.type.new()
		ev.shift_pressed = a.get("shift") || false
		ev.alt_pressed = a.get("alt") || false 
		ev.ctrl_pressed = a.get("ctrl") || false
		ev.meta_pressed = a.get("cmd") || false
		ev.command_or_control_autoremap = a.get("meta") || false
		if a.type == InputEventMouseButton:
			ev.button_index = a.btn_idx
		elif a.type == InputEventKey:
			ev.set_keycode(a.btn_idx)
		InputMap.action_add_event(a.name, ev)
