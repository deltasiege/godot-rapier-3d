extends MultiplayerSpawner

@export var player_scene: PackedScene

func spawn_all_players():
	if not multiplayer.is_server(): return
	multiplayer.peer_connected.connect(add_player)
	multiplayer.peer_disconnected.connect(del_player)
	
	# Spawn other peer players
	var peers = multiplayer.get_peers()
	for idx in peers.size():
		add_player(peers[idx], get_spawn_pos(idx + 1))

	# Spawn the server's local player unless this is a dedicated server export.
	if not OS.has_feature("dedicated_server"): add_player(1, get_spawn_pos(0))

func get_spawn_pos(idx: int):
	return Vector3((2 * idx), 2, 0)

func _exit_tree():
	if not multiplayer.is_server(): return
	multiplayer.peer_connected.disconnect(add_player)
	multiplayer.peer_disconnected.disconnect(del_player)

func add_player(id: int, pos: Vector3):
	var player = player_scene.instantiate()
	player.peer_id = id
	player.spawn_pos = pos
	player.name = str(id)
	add_child(player, true)
	player.owner = self

func del_player(id: int):
	if not has_node(str(id)): return
	get_node(str(id)).queue_free()
