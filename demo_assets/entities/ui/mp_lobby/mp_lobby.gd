extends Control

@onready var ip = $"Peer Type"/MarginContainer/VBoxContainer/GridContainer/IP
@onready var port = $"Peer Type"/MarginContainer/VBoxContainer/GridContainer/Port
@onready var player_list = $Lobby/VBoxContainer/MarginContainer/VBoxContainer/Players

@onready var start_btn = $"Lobby/VBoxContainer/MarginContainer/VBoxContainer/Start"
@onready var peer_type = $"Peer Type"
@onready var lobby = $Lobby

signal host_pressed(port: int)
signal join_pressed(ip: String, port: int)
signal back_pressed
signal start_pressed

func _ready(): _switch_panel(peer_type)

func add_self(): _add_peer(multiplayer.get_unique_id(), true)
func add_peer(id: int): _add_peer(id, false)

func _add_peer(id: int, is_me: bool):
	var label = Label.new()
	var prefix = "Me #" if is_me else "Peer #"
	label.text = prefix + str(id)
	label.name = str(id)
	player_list.add_child(label)
	label.owner = player_list

func remove_peer(id: int):
	var found = player_list.find_child(str(id), false, false)
	if found: found.queue_free()
	else: push_error("Could not find peer to remove from Lobby UI: ", id)

func remove_all_peers():
	for entry in player_list.get_children(): entry.queue_free()

func _hide_all_panels():
	var panels = [peer_type, lobby]
	for p in panels: p.visible = false

func _switch_panel(panel: Control):
	_hide_all_panels()
	panel.visible = true

func _on_host_pressed(): 
	start_btn.visible = true
	_switch_panel(lobby)
	emit_signal("host_pressed", int(port.text))

func _on_join_pressed():
	start_btn.visible = false
	_switch_panel(lobby)
	emit_signal("join_pressed", ip.text, int(port.text))

func _on_start_pressed():
	_hide_all_panels()
	emit_signal("start_pressed")

func _on_back_pressed():
	_switch_panel(peer_type)
	remove_all_peers()
	emit_signal("back_pressed")
