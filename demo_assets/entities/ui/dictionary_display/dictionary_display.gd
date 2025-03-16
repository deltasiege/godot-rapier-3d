extends Control

## Displays dictionary key + value pairs in a vertical list

@export var container: Control = self
@export var text_entry_scene: PackedScene = preload("./data_entry.tscn")
@export var theme_res: Theme = preload("../theme.tres")

var _merge_data = { "container": container, "theme": theme_res, "text_entry_scene": text_entry_scene }

func _ready(): theme = theme_res

func create_entries(entries):
	if entries is Array: for entry: Dictionary in entries: create_entry(entry.get("type"), entry.merged(_merge_data))
	elif entries is Dictionary: for key in entries: create_entry("text", { "text": key, "value": str(entries[key]) }.merged(_merge_data))

func set_entries(entries, sync: bool = true):
	if entries is Array: for entry in entries: set_entry(entry.get("type"), entry.merged(_merge_data))
	elif entries is Dictionary: for key in entries: set_entry("text", { "text": key, "value": str(entries[key]) }.merged(_merge_data))
	
	if !sync: return
	delete_extra_entries(entries)
	add_missing_entries(entries)

func create_entry(type, data: Dictionary):
	match type:
		"button": create_button_entry(data)
		"text": create_text_entry(data)
		_: create_text_entry(data)

func set_entry(type, data: Dictionary):
	match type:
		"button": set_button_entry(data)
		"text": set_text_entry(data)
		_: set_text_entry(data)

func force_set_entries(entries):
	clear_entries()
	create_entries(entries)

func clear_entries():
	for child in container.get_children(): child.queue_free()

func delete_extra_entries(desired_entries):
	for child in container.get_children():
		if desired_entries is Array and desired_entries.filter(func(entry): return entry.get("id", entry.get("text")) == child.name).size() <= 0: child.queue_free()
		elif desired_entries is Dictionary and !desired_entries.keys().has(child.name): child.queue_free()

func add_missing_entries(desired_entries):
	if desired_entries is Array:
		for entry: Dictionary in desired_entries:
			var id = entry.get("id", entry.get("text"))
			if container.find_child(id, false) != null: continue
			create_entry(entry.get("type"), entry.merged(_merge_data))
	elif desired_entries is Dictionary:
		for key in desired_entries:
			if container.find_child(key, false) != null: continue
			create_entry("text", { "text": key, "value": str(desired_entries[key]) }.merged(_merge_data))

static func create_button_entry(data: Dictionary):
	var id = str(data.get("id", data.text))
	var new_button = Button.new()
	new_button.text = str(data.text)
	new_button.name = id
	new_button.connect("pressed", data.on_pressed)
	new_button.action_mode = BaseButton.ACTION_MODE_BUTTON_PRESS
	data.container.add_child(new_button)
	new_button.owner = data.container

static func set_button_entry(data: Dictionary):
	var id = str(data.get("id", data.text))
	var found_button: Button = data.container.find_child(id)
	if !found_button: return
	found_button.text = str(data.text)
	found_button.name = id
	for conn in found_button.get_signal_connection_list("pressed"):
		if conn.signal.get_name() == "pressed": conn.signal.disconnect(conn.callable)
	found_button.connect("pressed", data.on_pressed)

static func create_text_entry(data: Dictionary):
	var new_entry = data.text_entry_scene.instantiate()
	var label = new_entry.get_node("Label")
	var ledit = new_entry.get_node("LineEdit")
	new_entry.name = str(data.get("id", data.text))
	label.theme = data.theme
	ledit.theme = data.theme
	label.text = str(data.text)
	label.name = str(data.text) + "_label"
	ledit.text = str(data.value)
	ledit.name = str(data.text) + "_ledit"
	data.container.add_child(new_entry)
	new_entry.owner = data.container

static func set_text_entry(data: Dictionary):
	var found_label = data.container.find_child(str(data.text) + "_label")
	var found_ledit = data.container.find_child(str(data.text) + "_ledit")
	if !found_label or !found_ledit: return
	found_label.text = str(data.text)
	found_ledit.text = str(data.value)
