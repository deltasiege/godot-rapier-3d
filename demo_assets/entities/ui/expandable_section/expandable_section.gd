@tool
extends Control

@export_category("Button")
@export var title := "Title":
	set(value): set_title(value); title = value
@export var title_label: Label
@export var stretch_button := false:
	set(value): set_stretch_button(value); stretch_button = value

@export_category("Rotating button icon")
@export var rot_target: Control
@export var rot_pivot := Vector2(0.5, 0.5)
@export var open_rotation := 0
@export var closed_rotation := 90

@export_category("Content")
@export var content_mask: Control
@export var content_margin_container: MarginContainer
@export var content_vbox_container: VBoxContainer
@export var content_margin := 8:
	set(value): set_content_margin(value); content_margin = value

@export_category("Animation")
@export var interruptible := true
@export var tween_duration := 0.075

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
var _tween_fac = 0
var _state := State.CLOSED
var _on_toggle_callbacks = []

# Public ---

# Provide a callback whenever this section is opened/closed
# Receives 1 bool argument indicating open/closed state
func on_toggle(callable: Callable):
	_on_toggle_callbacks.append(callable)

# Call this to provide your own section content from a script
func replace_content(node: Control, reparent: bool = false):
	clear_content()
	append_content(node, reparent)

# Call this to append your own section content from a script
# Multiple items will stack vertically
func append_content(node: Control, reparent: bool = false):
	if !content_vbox_container: push_error("content_vbox_container is not set"); return
	if reparent: node.reparent(content_vbox_container)
	else: content_vbox_container.add_child(node)
	node.owner = content_vbox_container
	set_button_enabled(true)
	shrink_content_container()

func shrink_content_container():
	if !is_inside_tree(): return
	await get_tree().process_frame
	await get_tree().process_frame
	content_margin_container.size.y = 0
	await get_tree().process_frame
	await get_tree().process_frame
	content_margin_container.set_position(Vector2(0, 0))

# Call this function to delete all content and mark the section as empty
func clear_content():
	for child in content_vbox_container.get_children(): child.queue_free()
	set_button_enabled(false)

# Call this to open the section
func open(instant: bool = false):
	if _state == State.OPENED or cant_interrupt(_state): return
	set_rot_pivot()
	if instant: _state = State.OPENED; _tween_fac = 1; process_tween_fac()
	else: _state = State.OPENING

# Call this to close the section
func close(instant: bool = false):
	if _state == State.CLOSED or cant_interrupt(_state): return
	set_rot_pivot()
	if instant: _state = State.CLOSED; _tween_fac = 0; process_tween_fac()
	else: _state = State.CLOSING

func toggle():
	var new_state: bool
	match _state:
		State.OPENED, State.OPENING:
			new_state = false
			close()
		State.CLOSED, State.CLOSING:
			new_state = true
			open()
			
	for cb: Callable in _on_toggle_callbacks: cb.call(new_state)

# Private ---

func _ready():
	if missing_defs(): return
	if start_open and has_content(): open(true)
	else: close(true)
	setup_button()
	process_tween_fac()
	shrink_content_container()

func _process(delta):
	if missing_defs(): return
	watch_content_size()
	process_tweens(delta)

func watch_content_size():
	if _last_content_size != get_content_size(): # Content size change
		fit_mask_to_content(_tween_fac)
		shrink_content_container()
		_last_content_size = get_content_size()

func process_tweens(delta):
	if is_tweening(_state): # Tween
		var sign := 1
		match _state:
			State.OPENING:
				sign = 1
				if _tween_fac == 1: _state = State.OPENED
			State.CLOSING:
				sign = -1
				if _tween_fac == 0: _state = State.CLOSED
		_tween_fac = clampf(_tween_fac + (sign * (1 / tween_duration) * delta), 0, 1)
		process_tween_fac()

func process_tween_fac():
	fit_mask_to_content(_tween_fac)
	rotate_target(_tween_fac)

func set_title(new_title: String):
	if !title_label: return
	title_label.text = new_title

func set_rot_pivot():
	if !rot_target: return
	rot_target.pivot_offset = rot_target.size * rot_pivot

func setup_button():
	var btn = get_button()
	if !btn: return
	btn.connect("pressed", self.toggle)

func get_button() -> Button:
	var btn: Button = self.get_node("./ButtonMarginContainer/Button")
	return btn

func set_button_enabled(enabled: bool):
	var btn = get_button()
	if !btn: return
	btn.disabled = !enabled

func set_stretch_button(stretch: bool):
	var btn_container: MarginContainer = self.get_node("./ButtonMarginContainer")
	if !btn_container: return
	btn_container.size_flags_horizontal = Control.SIZE_FILL if stretch else Control.SIZE_SHRINK_BEGIN

func get_content_size() -> Vector2:
	return Vector2(content_margin_container.size.x, content_margin_container.size.y)

func fit_mask_to_content(tween_fac: float = 1):
	# UP TO - mask size needs to take into account that nested sections might be collapsed
	
	content_mask.custom_minimum_size.x = content_margin_container.size.x
	content_mask.custom_minimum_size.y = tween_fac * content_margin_container.size.y

func rotate_target(tween_fac: float = 1):
	if !rot_target: return
	rot_target.rotation = lerpf(deg_to_rad(open_rotation), deg_to_rad(closed_rotation), tween_fac)

func cant_interrupt(state: State) -> bool:
	return is_tweening(state) and !interruptible

func is_tweening(state: State) -> bool:
	return state == State.OPENING or state == State.CLOSING

func has_content() -> bool:
	return content_vbox_container.get_children().size() > 0

func set_content_margin(new_margin: int):
	if !content_margin_container: return
	content_margin_container.add_theme_constant_override("margin_left", new_margin)
	content_margin_container.add_theme_constant_override("margin_top", new_margin)
	content_margin_container.add_theme_constant_override("margin_right", new_margin)
	content_margin_container.add_theme_constant_override("margin_bottom", new_margin)

func missing_defs() -> bool:
	return content_mask == null or content_margin_container == null or content_vbox_container == null or _state == null
