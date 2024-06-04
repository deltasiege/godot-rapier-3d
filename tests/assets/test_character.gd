extends RapierCharacterBody3D


# Called when the node enters the scene tree for the first time.
func _ready():
	move_and_slide(Vector3.ONE, 0)
