# Adapted from https://github.com/adamviola/simple-free-look-camera/blob/master/camera.gd
extends Camera3D

@export_group("Movement")
@export_range(0.0, 100.0) var speed: float = 50
@export_range(0.0, 100.0) var fast_speed: float = 100
@export_range(0.0, 100.0) var slow_speed: float = 25
@export_range(0.0, 10.0) var sensitivity: float = 1

@export_group("Features")
@export var capture_mouse = true ## Lock and hide the mouse

@export_group("Input Map action names")
@export var fwd_action = "camera_forward" ## W key by default
@export var back_action = "camera_backward" ## S key by default
@export var left_action = "camera_left" ## A key by default
@export var right_action = "camera_right" ## D key by default
@export var up_action = "camera_up" ## Q key by default
@export var down_action = "camera_down" ## E key by default
@export var fast_action = "camera_fast" ## SHIFT key by default
@export var slow_action = "camera_slow" ## W ALT by default

var input_enabled = true
var is_moving = false
signal motion_changed ## Emits true signal when FlyCam starts moving, false when it stops moving

# Mouse state
var _mouse_position = Vector2.ZERO
var _total_pitch = 0.0

# Movement state
var _direction = Vector3.ZERO
var _velocity = Vector3.ZERO
var _acceleration = 30
var _deceleration = -10

var actions = [
	{ "name": fwd_action, "type": InputEventKey, "btn_idx": KEY_W },
	{ "name": back_action, "type": InputEventKey, "btn_idx": KEY_S },
	{ "name": left_action, "type": InputEventKey, "btn_idx": KEY_A },
	{ "name": right_action, "type": InputEventKey, "btn_idx": KEY_D },
	{ "name": up_action, "type": InputEventKey, "btn_idx": KEY_Q },
	{ "name": down_action, "type": InputEventKey, "btn_idx": KEY_E },
	{ "name": fast_action, "type": InputEventKey, "btn_idx": KEY_SHIFT },
	{ "name": slow_action, "type": InputEventKey, "btn_idx": KEY_ALT }
]

func _ready():
	ControlsMgr.add_input_map_actions(actions)
	#if capture_mouse: Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _process(delta):
	if !input_enabled: return
	if !WindowMgr.focused: return
	_update_mouselook()
	_update_movement(delta)

func _input(event):
	if event is InputEventMouseMotion: _mouse_position = event.relative / (get_viewport().size.x / 100) # Store mouse movement

func toggle_input(enabled: bool):
	input_enabled = enabled

func action_to_float(action_name: String) -> float:
	return 1.0 if Input.is_action_pressed(action_name) else 0.0

func _get_speed():
	if Input.is_action_pressed(fast_action): return fast_speed
	elif Input.is_action_pressed(slow_action): return slow_speed
	else: return speed

func _update_movement(delta):
	_direction = Vector3(
		action_to_float(right_action) - action_to_float(left_action), 
		action_to_float(down_action) - action_to_float(up_action),
		action_to_float(back_action) - action_to_float(fwd_action)
	)
	
	var _was_moving = is_moving
	is_moving = _direction.length() > 0
	var started_moving = is_moving && !_was_moving
	var stopped_moving = !is_moving && _was_moving
	if started_moving || stopped_moving : motion_changed.emit(is_moving) 
	
	var _speed = _get_speed()
	var move_delta = _direction.normalized() * _acceleration * _speed * delta \
		+ _velocity.normalized() * _deceleration * _speed * delta
	
	if _direction == Vector3.ZERO and move_delta.length_squared() > _velocity.length_squared(): # Check if we should bother translating the camera
		_velocity = Vector3.ZERO # Set velocity to 0 to prevent jittering due to imperfect deceleration
	else: _velocity = Vector3(_velocity + move_delta).clampf(-_speed, _speed)
	
	translate(_velocity * delta)
	
	if projection == Camera3D.PROJECTION_ORTHOGONAL:
		size += _velocity.z / 100

func _update_mouselook():
	_mouse_position *= sensitivity
	var yaw = _mouse_position.x
	var pitch = _mouse_position.y
	_mouse_position = Vector2(0, 0)
	
	# Prevent looking up/down too far
	pitch = clamp(pitch, -90 - _total_pitch, 90 - _total_pitch)
	_total_pitch += pitch

	rotate_y(deg_to_rad(-yaw))
	rotate_object_local(Vector3(1,0,0), deg_to_rad(-pitch))
