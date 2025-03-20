extends Node

@export var player_scene: PackedScene
@export var players_container: Node
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
	spawn_all_players()

func spawn_all_players():
	spawn_player(multiplayer.get_unique_id(), Vector3(0, 2, 0))
	var peers = multiplayer.get_peers()
	for idx in peers.size(): spawn_player(peers[idx], Vector3((2 * idx) + 2, 2, 0))

func spawn_player(pid: int, pos: Vector3):
	var new_player = player_scene.instantiate()
	new_player.name = str(pid)
	new_player.set_multiplayer_authority(pid, false)
	var char: RapierKinematicCharacter3D = new_player.get_child(0)
	char.set_multiplayer_authority(pid, true)
	char.set_uid(GR3D.create_cuid())
	players_container.add_child(new_player)
	char.global_position = pos
	new_player.owner = players_container
