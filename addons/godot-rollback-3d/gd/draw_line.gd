extends MeshInstance3D

var _lines := []
var st: SurfaceTool
var color_override = null # Color(0.07, 0.93, 0.19)

func _ready():
	process_mode = Node.PROCESS_MODE_ALWAYS
	material_override = _get_line_material()
	st = SurfaceTool.new()

func _process(_delta):
	mesh = _lines_to_mesh(_lines, mesh, st, color_override)
	_lines.clear()

func draw_line(a: Vector3, b: Vector3, color: Color):
	_lines.append([a, b, color])

func draw_lines(lines: Array):
	for line in lines:
		var a = Vector3(line[0][0], line[0][1], line[0][2])
		var b = Vector3(line[1][0], line[1][1], line[1][2])
		var color = Color(line[2][0], line[2][1], line[2][2], line[2][3])
		_lines.append([a, b, color])

func clear_lines():
	_lines.clear()
	st.clear()
	mesh = null

static func _lines_to_mesh(lines: Array, mesh: Mesh, st: SurfaceTool, color):
	if !st: return
	st.clear()
	if lines.size() == 0:
		return null
	st.begin(Mesh.PRIMITIVE_LINES)
	for line in lines:
		var p1 : Vector3 = line[0]
		var p2 : Vector3 = line[1]
		var line_color : Color = line[2]
		var col = color if color != null else line_color
		st.set_color(col)
		st.add_vertex(p1)
		st.add_vertex(p2)
	return st.commit()

static func _get_line_material() -> StandardMaterial3D:
	var mat : StandardMaterial3D
	mat = StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.no_depth_test = false
	mat.render_priority = mat.RENDER_PRIORITY_MAX
	mat.vertex_color_use_as_albedo = true
	return mat
