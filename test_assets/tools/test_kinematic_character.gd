extends RapierKinematicCharacter3D

@export var speed = 10
@export var accel = 2
@export var decel = 1
@export var gravity = 120
@export var jump_gravity = 60
@export var jump_velocity = 25
@export var coyote_time_ms = 250 ## How many milliseconds late after falling off something the player can press jump and still get a jump
@export var lookat_pivots: Array[Node3D]

var velocity = Vector3.ZERO

var _airborne
var _gravity = gravity
var _last_grounded_ts = -INF

func simulate_inputs():
	if Engine.get_physics_frames() % 60 == 0: Input.action_press("jump")

func _physics_process(delta):
	simulate_inputs()
	
	var floored = is_on_floor()
	if not floored: velocity.y -= _gravity * delta # Gravity
	
	var jump_pressed = Input.is_action_just_pressed("jump")
	var time_since_grounded = Time.get_ticks_msec() - _last_grounded_ts
	var was_on_floor = time_since_grounded <= coyote_time_ms
	var can_jump = floored or was_on_floor
	var falling = velocity.y < 0
	if jump_pressed and can_jump and !_airborne: # Jumping
		_airborne = true
		velocity.y = jump_velocity
		_gravity = jump_gravity
	elif floored and _airborne: # Landing
		_airborne = false
		velocity.y = 0
	elif not floored and _airborne and falling: # Descending
		_gravity = gravity
	
	if floored: _last_grounded_ts = Time.get_ticks_msec()
	
	# Moving
	var input_dir = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	var direction = (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	if direction:
		velocity.x = move_toward(velocity.x, direction.x * speed, accel)
		velocity.z = move_toward(velocity.z, direction.z * speed, accel)
	else:
		velocity.x = move_toward(velocity.x, 0, decel)
		velocity.z = move_toward(velocity.z, 0, decel)
	move_by_amount(velocity * delta)
