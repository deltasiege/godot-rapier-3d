extends Node

@export var player_spawner: MultiplayerSpawner
@onready var ui = $"Lobby"

var players = {}
var num_players = 0

const MAX_CLIENTS = 6

func _ready():
	ui.connect("host_pressed", host)
	ui.connect("join_pressed", join)
	ui.connect("back_pressed", reset)
	ui.connect("start_pressed", start)
	multiplayer.connect("peer_connected", ui.add_peer)
	multiplayer.connect("peer_disconnected", ui.remove_peer)
	GR3D.connect("sync_started", on_sync_start)
	NetworkManager.on_ready(self)

func host(port: int):
	NetworkManager.start_server(port, self)
	ui.add_self()

func join(ip: String, port: int):
	NetworkManager.connect_to_server(ip, port, self)
	ui.add_self()

func reset(): NetworkManager.reset(self)

func start():
	if !multiplayer.is_server(): return
	NetworkManager.start_sync(self)

func on_sync_start():
	spawn_players()

func spawn_players():
	var peer_map = GR3D.get_peer_map()
	var xform = Transform3D.IDENTITY
	var offset = 5
	for idx in peer_map.size():
		var player_idx = idx + 1
		xform.origin.x += offset
		GR3D.spawn(
			player_idx,
			"Player " + str(player_idx),
			get_parent(),
			"res://demo_assets/entities/characters/mp_player.tscn",
			xform
		)
