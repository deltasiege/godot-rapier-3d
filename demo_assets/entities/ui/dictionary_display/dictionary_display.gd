extends Control

## Displays dictionary key + value pairs in a vertical list

@export var container: Control = self
@export var theme_res: Theme = preload("../theme.tres")

func _ready(): theme = theme_res

func create_entries(entries):
	var arr = homogenize_entries(entries)
	for entry in arr: create_entry(entry)

func set_entries(entries, sync: bool = true):
	var arr = homogenize_entries(entries)
	for entry in arr: set_entry(entry)
	if !sync: return
	delete_extra_entries(arr)
	add_missing_entries(arr)

func create_entry(entry: Dictionary):
	match entry.type:
		"text": create_text_entry(entry)
		"button": create_button_entry(entry)
		"group": create_group(entry)
		_: pass

func set_entry(entry: Dictionary):
	match entry.type:
		"text": set_text_entry(entry)
		"button": set_button_entry(entry)
		_: pass

func force_set_entries(entries):
	clear_entries()
	create_entries(entries)

func clear_entries():
	for child in container.get_children(): child.queue_free()

func delete_extra_entries(desired_entries: Array):
	for child in container.get_children():
		if desired_entries.filter(func(entry):
			return child.name.contains(str(entry.get("id", entry.get("key"))))
		).size() <= 0: child.queue_free()

func add_missing_entries(desired_entries: Array):
	for entry: Dictionary in desired_entries:
		var id = str(entry.get("id", entry.get("key")))
		if container.find_child(id, false, false) != null: continue
		create_entry(entry)

func create_button_entry(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = str(entry.get("id", key))
	var new_button = Button.new()
	new_button.text = key
	new_button.name = id
	new_button.theme = theme_res
	new_button.connect("pressed", entry.on_pressed)
	new_button.action_mode = BaseButton.ACTION_MODE_BUTTON_PRESS
	container.add_child(new_button)
	new_button.owner = container

func set_button_entry(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = str(entry.get("id", key))
	var found_button: Button = container.find_child(id)
	if !found_button: return
	found_button.text = key
	found_button.name = id
	for conn in found_button.get_signal_connection_list("pressed"):
		if conn.signal.get_name() == "pressed": conn.signal.disconnect(conn.callable)
	found_button.connect("pressed", entry.on_pressed)

func create_text_entry(entry: Dictionary):
	var hbox = HBoxContainer.new()
	var label = Label.new()
	var ledit = LineEdit.new()
	var key = str(entry.get("key", null))
	var id = str(entry.get("id", key))
	var value = str(entry.get("value", null))
	hbox.name = id
	label.theme = theme_res
	ledit.theme = theme_res
	label.text = key
	label.name = id + "_label"
	ledit.text = value
	ledit.name = id + "_ledit"
	ledit.editable = false
	ledit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.add_child(hbox)
	hbox.add_child(label)
	hbox.add_child(ledit)
	hbox.owner = container

func set_text_entry(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = str(entry.get("id", key))
	var value = str(entry.get("value", null))
	var found_hbox = container.find_child(id)
	if !found_hbox: return
	var found_label = found_hbox.find_child(id + "_label", false, false)
	var found_ledit = found_hbox.find_child(id + "_ledit", false, false)
	if !found_label or !found_ledit: return
	found_label.text = key
	found_ledit.text = value

func create_group(entry: Dictionary):
	var label = Label.new()
	var sep = HSeparator.new()
	var key = str(entry.get("key", null))
	var id = str(entry.get("id", key))
	label.name = id
	label.theme = theme_res
	label.text = key
	sep.name = id + "_sep"
	sep.theme = theme_res
	container.add_child(label)
	container.add_child(sep)
	label.owner = container
	sep.owner = container

func homogenize_entries(entries) -> Array:
	var flat = []
	var group_types = ["Dictionary"]
	if entries is Array: flat = entries
	else: flatten_dict(entries, flat)
	var out = []
	for entry: Dictionary in flat:
		var given_type = entry.type if entry.has("type") else null
		var inferred_type = type_string(typeof(entry.value)) if (entry.has("value") and entry.value != null) else "unknown"
		var type
		if group_types.has(given_type) or group_types.has(inferred_type): type = "group"
		elif given_type: type = given_type
		else: type = "text"
		var data = entry.merged({ "type": type }, true)
		out.append(data)
	return out

func flatten_dict(dict: Dictionary, out_arr: Array):
	for key in dict:
		if dict[key] is Dictionary:
			out_arr.append({ "type": type_string(typeof(dict)), "key": key })
			flatten_dict(dict[key], out_arr)
		else: out_arr.append({ "key": key, "value": dict[key] })
