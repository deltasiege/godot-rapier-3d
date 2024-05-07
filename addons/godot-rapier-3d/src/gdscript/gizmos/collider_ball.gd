extends EditorNode3DGizmoPlugin

func _get_gizmo_name():
	return "RapierCollider3D"

func _init():
	var settings = EditorInterface.get_editor_settings()
	var gizmo_color: Color = settings.get_setting("editors/3d_gizmos/gizmo_colors/shape")
	create_material("main", gizmo_color)
	create_handle_material("handles")

func _has_gizmo(node):
	return node is RapierCollider3D

func _redraw(gizmo):
	gizmo.clear()
	
	var lines = PackedVector3Array()
	var collider_node = gizmo.get_node_3d()
	var radius = 10.0
	
	for i in 360:
		var ra = deg_to_rad(float(i));
		var rb = deg_to_rad(float(i + 1));
		var a = Vector2(sin(ra), cos(ra)) * radius;
		var b = Vector2(sin(rb), cos(rb)) * radius;

		lines.push_back(Vector3(a.x, 0, a.y));
		lines.push_back(Vector3(b.x, 0, b.y));
		lines.push_back(Vector3(0, a.x, a.y));
		lines.push_back(Vector3(0, b.x, b.y));
		lines.push_back(Vector3(a.x, a.y, 0));
		lines.push_back(Vector3(b.x, b.y, 0));
	
	gizmo.add_lines(lines, get_material("main", gizmo), false)
	
	#var handles = PackedVector3Array()
	#handles.push_back(Vector3(0, 1, 0))
	#handles.push_back(Vector3(0, 5, 0))
	#gizmo.add_handles(handles, get_material("handles", gizmo), [])
