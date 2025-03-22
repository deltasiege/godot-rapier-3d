extends Node

@export var player_spawner: MultiplayerSpawner
@onready var ui = $"P2P Lobby"

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
	P2PNet.connect_console(self)

func host(port: int):
	P2PNet.host(port, self)
	ui.add_self()

func join(ip: String, port: int):
	P2PNet.join(ip, port, self)
	ui.add_self()

func reset(): P2PNet.reset(self)

func start():
	if !multiplayer.is_server(): return
	player_spawner.spawn_all_players()
