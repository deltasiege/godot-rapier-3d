static func _append_children_by_class(_class: String, node: Node, arr: Array):
	if node.is_class(_class): arr.append(node)
	for child in node.get_children(): _append_children_by_class(_class, child, arr)
