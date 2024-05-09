extends MeshInstance3D

var _line_material_pool := []
var _lines := []
var st
var color_override = Color(0, 0.5, 0.5)

func _ready():
	process_mode = Node.PROCESS_MODE_ALWAYS
	material_override = _get_line_material()
	st = SurfaceTool.new()
	

func draw_line(a: Vector3, b: Vector3, color: Color):
	_lines.append([a, b, color])

func _get_line_material() -> StandardMaterial3D:
	var mat : StandardMaterial3D
	if len(_line_material_pool) == 0:
		mat = StandardMaterial3D.new()
		mat.flags_unshaded = true
		mat.no_depth_test = false
		mat.render_priority = mat.RENDER_PRIORITY_MAX
		mat.vertex_color_use_as_albedo = true
	else:
		mat = _line_material_pool[-1]
		_line_material_pool.pop_back()
	return mat

func _process(_delta: float):
	color_override = Color(0.5, 0.97, 0.55)
	if !st: return
	material_override = _get_line_material()
	st.clear()
	if _lines.size() == 0: 
		mesh = null
		return
	st.begin(Mesh.PRIMITIVE_LINES)
	for line in _lines:
		var p1 : Vector3 = line[0]
		var p2 : Vector3 = line[1]
		var color : Color = line[2]
		if color_override != null: st.set_color(color_override)
		else: st.set_color(color)
		st.add_vertex(p1)
		st.add_vertex(p2)
	mesh = st.commit()
	_lines.clear()
