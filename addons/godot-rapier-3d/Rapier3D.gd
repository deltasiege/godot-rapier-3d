extends Node3D

var debug := true

func _ready():
	var root = get_tree().get_current_scene()
	var pipeline = _get_pipeline(root)
	if debug: _print_counts(pipeline)

func _get_pipeline(root_node: Node) -> RapierPhysicsPipeline:
	var pipeline = RapierPhysicsPipeline.new()
	var rigid_bodies = _get_all_rigidbodies(root_node)
	for rigid_body in rigid_bodies:
		pipeline.add_rigid_body(rigid_body)
		var colliders = _get_colliders(rigid_body)
		for collider in colliders: pipeline.add_collider_with_parent(collider, rigid_body)
	return pipeline

func _get_colliders(rigid_body: RapierRigidBody3D):
	var colliders = []
	_append_children_with_class("RapierCollider3D", rigid_body, colliders)
	return colliders

func _get_all_rigidbodies(node: Node):
	var rigidbodies = []
	_append_children_with_class("RapierRigidBody3D", node, rigidbodies)
	return rigidbodies

func _append_children_with_class(_class: String, node: Node, arr: Array):
	if node.is_class(_class): arr.append(node)
	for child in node.get_children(): _append_children_with_class(_class, child, arr)

func _print_counts(pipeline):
	var rigid_bodies = pipeline.count_rigid_bodies()
	var colliders = pipeline.count_colliders()
	print("#RapierRigidBody3D: ", rigid_bodies, " #RapierCollider3D ", colliders)
