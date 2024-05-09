extends MeshInstance3D

var _line_material_pool := []
var _lines := []
var _line_immediate_mesh : ImmediateMesh

func _ready():
	process_mode = Node.PROCESS_MODE_ALWAYS
	_line_immediate_mesh = ImmediateMesh.new()
	material_override = _get_line_material()
	mesh = _line_immediate_mesh

func draw_line(a: Vector3, b: Vector3, color: Color):
	_lines.append([a, b, color])

func _get_line_material() -> StandardMaterial3D:
	var mat : StandardMaterial3D
	if len(_line_material_pool) == 0:
		mat = StandardMaterial3D.new()
		mat.flags_unshaded = true
		mat.no_depth_test = true
		mat.render_priority = mat.RENDER_PRIORITY_MAX
		mat.grow = 5.0
		mat.vertex_color_use_as_albedo = true
	else:
		mat = _line_material_pool[-1]
		_line_material_pool.pop_back()
	return mat

func _process(_delta: float):
	var im := _line_immediate_mesh
	im.clear_surfaces()
	if _lines.size() == 0: return
	im.surface_begin(Mesh.PRIMITIVE_LINES)
	for line in _lines:
		var p1 : Vector3 = line[0]
		var p2 : Vector3 = line[1]
		var color : Color = line[2]
		#im.surface_set_color(color)
		im.surface_set_color(Color(1, 1, 0, 1))
		im.surface_add_vertex(p1)
		im.surface_add_vertex(p2)
	
	im.surface_end()
	_lines.clear()
