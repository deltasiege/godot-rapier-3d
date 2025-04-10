extends GR3DInputAdapter

## Required! A function that returns an array of all actions
##  that can be used by rollback nodes. Must not change at runtime.
func get_input_list() -> Array[String]:
	return \
	[
		"move",
		"jump"
	]

## Required! A function that returns some value for every possible
## action specified in all_inputs (specified by input_key).
func get_input(input_key: String) -> Variant:
	match input_key:
		"move": return Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
		"jump": return Input.is_action_just_pressed("jump")
		_: push_error("Unknown input_key: ", input_key); return null
