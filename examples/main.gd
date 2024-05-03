extends Node3D

@onready var pipe = RapierPhysicsPipeline.new()

func _ready():
	print(pipe)

func _physics_process(_delta):
	pipe.step()
