extends GR3DInputAdapter

## Required!
## A function that returns an array of all actions
## that can be used by rollback nodes. Must not change at runtime.
func get_input_list() -> Array[String]:
	return \
	[
		"move",
		"jump"
	]

## Required!
## A function that returns some value for every possible
## action specified in all_inputs (passed in as input_key).
func get_input(input_key: String, node_state: Dictionary) -> Variant:
	match input_key:
		"move": 
			var input_dir = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
			var cam_relative_dir = (node_state.cam_dir * Vector3(input_dir.x, 0, input_dir.y)).normalized()
			return cam_relative_dir
		"jump": return Input.is_action_just_pressed("jump")
		_: push_error("Unknown input_key: ", input_key); return null

## Required!
## A function that returns a default value for every possible
## action specified in all_inputs (passed in as input_key).
func get_default(input_key: String) -> Variant:
	match input_key:
		"move": return Vector2.ZERO
		"jump": return false
		_: push_error("Unknown input_key: ", input_key); return null

## Optional
## Given a previous known input, what should we predict the next input to be?
## If not provided, the prediction defaults to repeating the previous known input.
##
## This might make sense for e.g. movement inputs - but for button presses like jump,
## it makes more sense to assume a player will not press and repress the jump input
## within a single frame frame, so we predict false instead.
func get_predicted_input(input_key: String, previous_input: Variant) -> Variant:
	match input_key:
		"move": return previous_input
		"jump": return false
		_: push_error("Unknown input_key: ", input_key); return null
