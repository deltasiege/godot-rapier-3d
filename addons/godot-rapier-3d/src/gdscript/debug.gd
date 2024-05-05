#static var debug := true
#
#static func _print_counts(pipeline: RapierPhysicsPipeline):
	#if !debug: return
	#var rigid_bodies = pipeline.count_rigid_bodies()
	#var colliders = pipeline.count_colliders()
	#print("#RapierRigidBody3D: ", rigid_bodies, " #RapierCollider3D ", colliders)
