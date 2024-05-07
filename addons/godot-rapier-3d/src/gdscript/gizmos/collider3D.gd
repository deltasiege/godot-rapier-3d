extends EditorNode3DGizmoPlugin

func _get_gizmo_name():
	return "RapierCollider3D"

func _init():
	var settings = EditorInterface.get_editor_settings()
	var gizmo_color: Color = settings.get_setting("editors/3d_gizmos/gizmo_colors/shape")
	create_material("main", gizmo_color)

func _has_gizmo(node):
	return (node is RapierCollider3D) and (node.get_class() == _get_gizmo_name())

func _redraw(gizmo: EditorNode3DGizmo):
	var node = gizmo.get_node_3d()
	if !_has_gizmo(node): return
	gizmo.clear()
	
	var lines
	match node.shape:
		"Ball":
			lines = Rapier3DGizmoShapes.ball(node.ball_radius)
		"Cuboid":
			lines = Rapier3DGizmoShapes.cuboid(node.cuboid_half_extents)
	
	gizmo.add_lines(lines, get_material("main", gizmo), false)
