@tool
extends Control

@export_category("Content")
@export var content_mask: Control
@export var content_container: MarginContainer
@export var content_margin := 8:
	get: return content_margin
	set(value): set_content_margin(value); content_margin = value

@export_category("Animation")
@export var interruptible := true
@export var tween_duration := 0.075

@export_category("Rotating button icon")
@export var rot_target: Control
@export var rot_pivot := Vector2(0.5, 0.5)
@export var open_rotation := 0
@export var closed_rotation := 90

@export_category("Test in editor")
@export_tool_button("Open", "Callable") var open_button = open
@export_tool_button("Close", "Callable") var close_button = close
@export_storage var start_open := true

enum State {
	OPENED,
	CLOSED,
	OPENING,
	CLOSING,
}

var _last_content_size = Vector2.ZERO
var _tween_progress = 1 if start_open else 0
var _state := State.OPENED if start_open else State.CLOSED

# Public ---

# Call this to provide your own section content from a script
func replace_content(node: Control):
	if !content_container: push_error("content_container is not set"); return
	for child in content_container.get_children(): child.queue_free()
	content_container.add_child(node)
	node.owner = content_container

# Call this to open the section
func open():
	if _state == State.OPENED or cant_interrupt(_state): return
	set_rot_pivot()
	_state = State.OPENING

# Call this to close the section
func close():
	if _state == State.CLOSED or cant_interrupt(_state): return
	set_rot_pivot()
	_state = State.CLOSING

# Private ---

func _ready():
	if missing_defs(): return
	if start_open: open()
	else: close()

func _process(delta):
	if missing_defs(): return
	watch_content_size()
	process_tweens(delta)

func watch_content_size():
	if _last_content_size != get_content_size(): # Content size change
		fit_mask_to_content(_tween_progress)
		_last_content_size = get_content_size()

func process_tweens(delta):
	if is_tweening(_state): # Tween
		var sign := 1
		match _state:
			State.OPENING:
				sign = 1
				if _tween_progress == 1: _state = State.OPENED
			State.CLOSING:
				sign = -1
				if _tween_progress == 0: _state = State.CLOSED
		_tween_progress = clampf(_tween_progress + (sign * (1 / tween_duration) * delta), 0, 1)
		fit_mask_to_content(_tween_progress)
		rotate_target(_tween_progress)

func set_rot_pivot():
	if !rot_target: return
	rot_target.pivot_offset = rot_target.size * rot_pivot

func get_content_size() -> Vector2:
	return Vector2(content_container.size.x, content_container.size.y)

func fit_mask_to_content(tween_progress: float = 1):
	content_mask.custom_minimum_size.x = content_container.size.x
	content_mask.custom_minimum_size.y = tween_progress * content_container.size.y

func rotate_target(tween_progress: float = 1):
	if !rot_target: return
	rot_target.rotation = lerpf(deg_to_rad(open_rotation), deg_to_rad(closed_rotation), tween_progress)

func cant_interrupt(state: State) -> bool:
	return is_tweening(state) and !interruptible

func is_tweening(state: State) -> bool:
	return state == State.OPENING or state == State.CLOSING

func set_content_margin(new_margin: int):
	if !content_container: return
	content_container.add_theme_constant_override("margin_left", new_margin)
	content_container.add_theme_constant_override("margin_top", new_margin)
	content_container.add_theme_constant_override("margin_right", new_margin)
	content_container.add_theme_constant_override("margin_bottom", new_margin)

func missing_defs() -> bool:
	return content_mask == null or content_container == null or _state == null
