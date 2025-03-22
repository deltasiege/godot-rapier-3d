extends Node

var focused := true
signal focus_changed(focused: bool)

func _notification(what: int) -> void:
	match what:
		NOTIFICATION_APPLICATION_FOCUS_OUT: focused = false
		NOTIFICATION_APPLICATION_FOCUS_IN: focused = true
	
	if what == NOTIFICATION_APPLICATION_FOCUS_IN || what == NOTIFICATION_APPLICATION_FOCUS_OUT:
		focus_changed.emit(focused)

func _input(event):
	if event is InputEventKey and event.keycode == KEY_ESCAPE:
		set_mouse_captured(false)

func set_mouse_mode(mouse_mode: Input.MouseMode):
	Input.set_mouse_mode(mouse_mode)

func toggle_mouse_confined():
	set_mouse_confined(!mouse_is_confined())

func toggle_mouse_captured():
	set_mouse_captured(!mouse_is_captured())

func set_mouse_confined(confined: bool):
	set_mouse_mode(Input.MOUSE_MODE_CONFINED if confined else Input.MOUSE_MODE_VISIBLE)

func set_mouse_captured(captured: bool):
	set_mouse_mode(Input.MOUSE_MODE_CAPTURED if captured else Input.MOUSE_MODE_VISIBLE)

func mouse_is_confined() -> bool:
	return Input.mouse_mode == Input.MOUSE_MODE_CONFINED

func mouse_is_captured() -> bool:
	return Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
