extends EditorNode3DGizmoPlugin

func _get_gizmo_name():
	return "RapierCollider3D"

func _init():
	var settings = EditorInterface.get_editor_settings()
	var gizmo_color: Color = settings.get_setting("editors/3d_gizmos/gizmo_colors/shape")
	create_material("main", gizmo_color)
	create_handle_material("handles")

func _has_gizmo(node):
	return (node is RapierCollider3D) and (node.get_class() == _get_gizmo_name())

func _get_handle_name(gizmo: EditorNode3DGizmo, handle_id: int, secondary: bool):
	var node = gizmo.get_node_3d()
	if !_has_gizmo(node): return
	match node.shape:
		"Ball": return "radius"

func _get_handle_value(gizmo: EditorNode3DGizmo, handle_id: int, secondary: bool):
	var node = gizmo.get_node_3d()
	if !_has_gizmo(node): return
	match node.shape:
		"Ball": return node.ball_radius

func _commit_handle(gizmo: EditorNode3DGizmo, handle_id: int, secondary: bool, restore: Variant, cancel: bool):
	var node = gizmo.get_node_3d()
	if !_has_gizmo(node): return
	match node.shape:
		"Ball":
			if cancel: node.ball_radius = restore
			var undo_redo = UndoRedo.new()
			undo_redo.create_action("Modify ball_radius")
			undo_redo.add_do_method(func(): node.ball_radius = node.ball_radius)
			undo_redo.add_undo_method(func(): node.ball_radius = restore)

func _redraw(gizmo: EditorNode3DGizmo):
	var node = gizmo.get_node_3d()
	if !_has_gizmo(node): return
	gizmo.clear()
	
	var lines
	var handles
	match node.shape:
		"Ball":
			lines = Rapier3DGizmoShapes.ball_lines(node.ball_radius)
			handles = Rapier3DGizmoShapes.ball_handles(node.ball_radius)
		"Cuboid":
			lines = PackedVector3Array()
	
	gizmo.add_lines(lines, get_material("main", gizmo), false)
	gizmo.add_handles(handles, get_material("handles", gizmo), [])
