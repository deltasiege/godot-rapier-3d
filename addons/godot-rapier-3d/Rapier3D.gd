extends Node3D

func say_foo():
	print("foo")

#const debug = preload("res://addons/godot-rapier-3d/src/gdscript/debug.gd")
#const rb = preload("res://addons/godot-rapier-3d/src/gdscript/rigidbody.gd")
#const utils = preload("res://addons/godot-rapier-3d/src/gdscript/utils.gd")

# var pipeline

# func _ready():
# 	var root = get_tree().get_current_scene()
	#pipeline = _get_pipeline(root)
	#debug._print_counts(pipeline)
#
#func _physics_process(_delta):
	#pipeline.step()
	#pipeline.sync_active_body_godot_transforms()
	#pass
#
#func _get_pipeline(root_node: Node) -> RapierPhysicsPipeline:
	#var pipeline = RapierPhysicsPipeline.new()
	#var rigid_bodies = rb._get_all_rigidbodies(root_node)
	#for rigid_body in rigid_bodies:
		#pipeline.add_rigid_body(rigid_body)
		#var colliders = _get_colliders(rigid_body)
		#for collider in colliders: pipeline.add_collider_with_parent(collider, rigid_body)
	#return pipeline
#
#func _get_colliders(rigid_body: RapierRigidBody3D):
	#var colliders = []
	#utils._append_children_with_class("RapierCollider3D", rigid_body, colliders)
	#return colliders
