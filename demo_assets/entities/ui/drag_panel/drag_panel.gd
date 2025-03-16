extends PanelContainer

@export var title: String = "Title"
@export var content_scene: PackedScene

@onready var drag_handle: Button = $VBoxContainer/HBoxContainer/Drag
@onready var content_container: Control = $VBoxContainer/MarginContainer

signal closed

var _hovering := false
var _dragging := false
var _drag_start := Vector2.ZERO
var current_content: Control = null

func _ready():
	drag_handle.text = title
	if content_scene != null: create_content()

func create_content(scene: PackedScene = content_scene):
	var content = scene.instantiate()
	content_container.add_child(content)
	content.owner = content_container
	current_content = content

func append_content(control: Control):
	if control.get_parent() != null: control.reparent(content_container)
	else: content_container.add_child(control)
	control.owner = content_container
	current_content = control

func _input(event):
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.is_pressed() and _hovering and !_dragging:
			_dragging = true
			var mouse_pos = get_viewport().get_mouse_position()
			_drag_start = position - mouse_pos
		elif !event.is_pressed() and _dragging:
			_dragging = false
	elif event is InputEventMouseMotion:
		if _dragging:
			var mouse_pos = get_viewport().get_mouse_position()
			position = mouse_pos + _drag_start

func close():
	emit_signal("closed")
	queue_free()

func _on_drag_mouse_entered(): _hovering = true
func _on_drag_mouse_exited(): _hovering = false
func _on_close_pressed(): close()
