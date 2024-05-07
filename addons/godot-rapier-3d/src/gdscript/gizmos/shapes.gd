class_name Rapier3DGizmoShapes

static func ball(radius: float = 1.0) -> PackedVector3Array:
	var lines = PackedVector3Array()
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
	return lines

static func cuboid(half_extents: Vector3 = Vector3(1.0, 1.0, 1.0)) -> PackedVector3Array:
	var lines = PackedVector3Array()
	var x = half_extents.x
	var y = half_extents.y
	var z = half_extents.z
	lines.push_back(Vector3(-x, -y, -z))
	lines.push_back(Vector3(x, -y, -z))
	lines.push_back(Vector3(-x, y, -z))
	lines.push_back(Vector3(x, y, -z))
	lines.push_back(Vector3(-x, -y, z))
	lines.push_back(Vector3(x, -y, z))
	lines.push_back(Vector3(-x, y, z))
	lines.push_back(Vector3(x, y, z))
	lines.push_back(Vector3(-x, -y, -z))
	lines.push_back(Vector3(-x, y, -z))
	lines.push_back(Vector3(x, -y, -z))
	lines.push_back(Vector3(x, y, -z))
	lines.push_back(Vector3(-x, -y, z))
	lines.push_back(Vector3(-x, y, z))
	lines.push_back(Vector3(x, -y, z))
	lines.push_back(Vector3(x, y, z))
	lines.push_back(Vector3(-x, -y, -z))
	lines.push_back(Vector3(-x, -y, z))
	lines.push_back(Vector3(x, -y, -z))
	lines.push_back(Vector3(x, -y, z))
	lines.push_back(Vector3(-x, y, -z))
	lines.push_back(Vector3(-x, y, z))
	lines.push_back(Vector3(x, y, -z))
	lines.push_back(Vector3(x, y, z))
	return lines
