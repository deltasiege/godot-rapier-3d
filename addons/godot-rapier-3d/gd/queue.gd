## This queue tries to ensure that actions requested of Rapier on each frame
## occur in the same order across different machines

# TODO DELME

#static func get_cuid(node: Node3D) -> String:
	#var cuid = node.get_meta("cuid", false)
	#if !cuid: 
		#push_error("[GR3D][queue]: Could not retrieve cuid of '" + node.name + "'. Please run 'addons/godot-rapier-3d/gd/fixup_cuids.gd' to resolve.")
		#return ""
	#return cuid
