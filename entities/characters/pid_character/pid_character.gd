extends RapierPIDCharacter3D

@export var speed = 10
@export var accel = 100
@export var decel = 1
@export var gravity = 65
@export var jump_gravity = 45
@export var jump_velocity = 15
@export var coyote_time_ms = 250 ## How many milliseconds late after falling off something the player can press jump and still get a jump
@export var lookat_pivots: Array[Node3D]

@onready var cam_pivot = $"3rdPersonCam"

var velocity = Vector3.ZERO

var _airborne
var _gravity = gravity
var _last_grounded_ts = -INF
var _last_jump_ts = -INF
var _jump_restick_threshold = 50

func _physics_process(delta):
	var floored = is_on_floor()
	if not floored:
		velocity.y -= _gravity * delta # Gravity
		
	var time_since_jump = Time.get_ticks_msec() - _last_jump_ts
	var recent_jump = time_since_jump <= _jump_restick_threshold
	if floored and not recent_jump: 
		_last_grounded_ts = Time.get_ticks_msec()
		velocity.y = 0
	
	var jump_pressed = Input.is_action_just_pressed("jump")
	var time_since_grounded = Time.get_ticks_msec() - _last_grounded_ts
	var was_on_floor = time_since_grounded <= coyote_time_ms
	var can_jump = floored or was_on_floor
	var falling = velocity.y < 0
	if jump_pressed and can_jump and !_airborne: # Jumping
		_airborne = true
		velocity.y = jump_velocity
		_gravity = jump_gravity
		_last_jump_ts = Time.get_ticks_msec()
	elif floored and _airborne: # Landing
		_airborne = false
	elif not floored and _airborne and falling: # Descending
		_gravity = gravity
	
	# Moving
	var input_dir = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	var direction = (cam_pivot.transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	if direction:
		velocity.x = move_toward(velocity.x, direction.x * speed, accel)
		velocity.z = move_toward(velocity.z, direction.z * speed, accel)
	else:
		velocity.x = move_toward(velocity.x, 0, decel)
		velocity.z = move_toward(velocity.z, 0, decel)
	
	move_by_amount(velocity * delta)
	
	look_at_travel_dir(self) # Looking

static func look_at_travel_dir(character: RapierPIDCharacter3D, ignore_y: bool = true):
	var vel = character.get_real_velocity()
	var dir = vel.normalized()
	var pos = character.global_position
	var target = pos - dir
	var pivots = character.lookat_pivots if character.get("lookat_pivots") != null else [character]
	if ignore_y:
		target.y = pos.y
		dir.y = 0
	var valid = dir != Vector3.ZERO and pos != target and vel.length() > 1
	if valid:
		for pivot in pivots:
			if pivot.global_position.distance_to(target) < 0.1: continue
			else: pivot.look_at(target)
