extends MultiplayerSynchronizer
class_name CharacterInputProvider

@export var controller: Node3D
@export var velocity := Vector3()

func _ready():
	set_process(get_multiplayer_authority() == multiplayer.get_unique_id()) # Only process for the local player.

func _process(delta):
	velocity = controller.velocity
