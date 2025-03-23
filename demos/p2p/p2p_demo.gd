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
	GR3DNet.on_ready(self)

func host(port: int):
	GR3DNet.start_server(port, self)
	ui.add_self()

func join(ip: String, port: int):
	GR3DNet.connect_to_server(ip, port, self)
	ui.add_self()

func reset(): GR3DNet.reset(self)

func start():
	if !multiplayer.is_server(): return
	GR3DNet.start_sync(self)
	await get_tree().create_timer(2.1).timeout
	player_spawner.spawn_all_players()
