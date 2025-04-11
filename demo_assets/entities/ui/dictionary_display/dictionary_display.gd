extends Control

## Displays dictionary key + value pairs in a vertical list
## Depends on the Expandable Section ui element for displaying nested dictionaries

@export var container: Control = self
@export var expandable_section: PackedScene
@export var theme_res: Theme = preload("../theme.tres")

var _last_entries_count: int
var _opened_groups = {}
var _existing_ids = []

# Public ---

## Converts the given dictionary's items to entries and then displays them
## Does not support dynamic entry types such as buttons
## Example input:
## { "my_key": "my_value", "my_group": { "nested_key": "nested_value" } }
func display_data(data: Dictionary):
	var output = convert_data_to_entries(data)
	var total_entries = output[0]
	var entries = output[1]
	create_or_set_entries(entries, total_entries)

## Allows utilizing "type" field to specify dynamic entries such as buttons
## Example `entries` input:
## [
##   {
##     "type": "group",
##     "key": "my_group",
##     "value": [
##       {
##         "type": "button",
##         "key": "Do something",
##         "on_pressed": "do_something"
##       },
##       {
##         "type": "text",
##         "key": "my_key",
##         "value": "my_value"
##       }
##     ]
##   }
## ]
func display_entries(entries: Array):
	var total_entries = count_entries(entries)
	create_or_set_entries(entries, total_entries)

# Recreate all entries if there has been a change in their number
func create_or_set_entries(entries: Array, total_entries: int):
	if total_entries != _last_entries_count:
		clear_children()
		await get_tree().process_frame # Wait one frame so Godot doesn't increment conflicting names
		create_from_entries(entries)
	else:
		set_from_entries(entries)
	_last_entries_count = total_entries

func create_from_entries(entries: Array):
	for entry: Dictionary in entries: create_entry(entry, container)

## Set entries without recreating them unnecessarily
func set_from_entries(entries: Array):
	for entry: Dictionary in entries: set_entry(entry)

# Private ---

func _ready(): theme = theme_res

func convert_data_to_entries(data: Dictionary) -> Array:
	var entries = []
	var counter = []
	_existing_ids.clear()
	recurse_data(data, entries, counter)
	var total_entries = counter.size()
	return [total_entries, entries]

func count_entries(entries: Array) -> int:
	var arr_counter = []
	var recurse := func(ents: Array, counter: Array, cb: Callable):
		for entry: Dictionary in ents:
			counter.append(0)
			if get_entry_type(entry) == "group" and entry.value is Array:
				cb.call(entry.value, counter, cb)
	recurse.call(entries, arr_counter, recurse)
	return arr_counter.size()

func recurse_data(data: Dictionary, out_arr: Array, out_arr_counter: Array):
	for key in data:
		if data[key] is Dictionary:
			var children = []
			recurse_data(data[key], children, out_arr_counter)
			out_arr_counter.append(0)
			out_arr.append({ "id": get_unique_id(str(key)), "type": "group", "key": key, "value": children })
		else:
			out_arr_counter.append(0)
			out_arr.append({ "id": get_unique_id(str(key)), "key": key, "value": data[key] })

func get_unique_id(key: String) -> String:
	var counter = 0
	while _existing_ids.has(key + "_" + str(counter)): counter += 1
	_existing_ids.append(key + "_" + str(counter))
	return key + "_" + str(counter)

var group_types = ["Dictionary"]
func get_entry_type(entry: Dictionary) -> String:
	var given_type = entry.type if entry.has("type") else null
	var inferred_type = type_string(typeof(entry.value)) if (entry.has("value") and entry.value != null) else "unknown"
	var type
	if group_types.has(given_type) or group_types.has(inferred_type): type = "group"
	elif given_type: type = given_type
	else: type = "text"
	return type

func create_entry(entry: Dictionary, parent: Control, parent_is_group: bool = false):
	var created_control: Control
	match get_entry_type(entry):
		"text": created_control = create_text(entry)
		"button": created_control = create_button(entry)
		"group": created_control = create_group(entry)
		_: pass
	
	var id = safe_name(str(entry.get("id", entry.get("key"))))
	if parent_is_group: parent.append_content(created_control)
	else: parent.add_child(created_control, true)
	created_control.name = id
	created_control.owner = parent

func set_entry(entry: Dictionary):
	match get_entry_type(entry):
		"text": set_text(entry)
		"button": set_button(entry)
		"group": set_group(entry)
		_: pass

func clear_children():
	for child in container.get_children(): child.queue_free()

func create_button(entry: Dictionary):
	var key = str(entry.get("key", null))
	var new_button = Button.new()
	new_button.text = key
	new_button.theme = theme_res
	new_button.connect("pressed", entry.on_pressed)
	new_button.action_mode = BaseButton.ACTION_MODE_BUTTON_PRESS
	return new_button

func set_button(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = safe_name(str(entry.get("id", key)))
	var found_button: Button = container.find_child(id, true, false)
	if !found_button: return
	found_button.text = key
	for conn in found_button.get_signal_connection_list("pressed"):
		if conn.signal.get_name() == "pressed": conn.signal.disconnect(conn.callable)
	found_button.connect("pressed", entry.on_pressed)

func create_text(entry: Dictionary):
	var hbox = HBoxContainer.new()
	var label = Label.new()
	var ledit = LineEdit.new()
	var key = str(entry.get("key", null))
	var id = safe_name(str(entry.get("id", key)))
	var value = str(entry.get("value", null))
	label.theme = theme_res
	ledit.theme = theme_res
	label.text = key
	label.name = id + "_label"
	ledit.text = value
	ledit.name = id + "_ledit"
	ledit.editable = false
	ledit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hbox.add_child(label)
	hbox.add_child(ledit)
	return hbox

func set_text(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = safe_name(str(entry.get("id", key)))
	var value = str(entry.get("value", null))
	var found_hbox = container.find_child(id, true, true)
	if !found_hbox: return
	var found_label = found_hbox.find_child(id + "_label", false, false)
	var found_ledit = found_hbox.find_child(id + "_ledit", false, false)
	if !found_label or !found_ledit: return
	found_label.text = key
	found_ledit.text = value

func create_group(entry: Dictionary):
	var new_group = expandable_section.instantiate()
	var key = str(entry.get("key", null))
	var id = safe_name(str(entry.get("id", key)))
	var has_sub_entries = entry.value.size() > 0
	new_group.on_toggle(record_group_toggle(id))
	for idx in entry.value.size(): 
		var sub_entry: Dictionary = entry.value[idx]
		if idx == 0: new_group.clear_content()
		create_entry(sub_entry, new_group, true)
	if has_sub_entries:
		new_group.start_open = group_was_open(id, true)
		new_group.title = key
	else:
		new_group.start_open = false
		new_group.clear_content(); new_group.title = key + " (empty)"
	return new_group

func set_group(entry: Dictionary):
	var key = str(entry.get("key", null))
	var id = safe_name(str(entry.get("id", key)))
	var found_group = container.find_child(id, true, false)
	if !found_group: return
	for idx in entry.value.size(): 
		var sub_entry: Dictionary = entry.value[idx]
		set_entry(sub_entry)
	if entry.value.size() == 0: found_group.title = key + " (empty)"
	else: found_group.title = key

func record_group_toggle(id: String) -> Callable:
	return func(new_state): _opened_groups[id] = new_state

func group_was_open(id: String, default: bool = false):
	if _opened_groups.has(id): return _opened_groups[id]
	return default

# Remove unsafe characters from name
func safe_name(unsafe_name: String) -> String:
	return unsafe_name.replace(":", "")
