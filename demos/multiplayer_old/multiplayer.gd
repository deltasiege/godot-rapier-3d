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
	await get_tree().create_timer(2.01).timeout
	player_spawner.spawn_all_players()
