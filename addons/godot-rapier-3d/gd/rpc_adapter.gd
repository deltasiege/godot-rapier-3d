extends GR3DNetworkAdapter

func _send_ping(peer_id: int, origin_time: String) -> void:
	_remote_ping.rpc_id(peer_id, origin_time)

@rpc("any_peer", "unreliable")
func _remote_ping(origin_time: String) -> void:
	var peer_id = multiplayer.get_remote_sender_id()
	received_ping.emit(peer_id, origin_time)

func _send_ping_back(peer_id: int, origin_time: String, local_time: String) -> void:
	_remote_ping_back.rpc_id(peer_id, origin_time, local_time)

@rpc("any_peer", "unreliable")
func _remote_ping_back(origin_time: String, local_time: String) -> void:
	var peer_id = multiplayer.get_remote_sender_id()
	received_ping_back.emit(peer_id, origin_time, local_time)

func _send_remote_start(peer_id: int) -> void:
	_remote_start.rpc_id(peer_id)

@rpc("any_peer")
func _remote_start() -> void:
	received_remote_start.emit()

func _send_remote_stop(peer_id: int) -> void:
	_remote_stop.rpc_id(peer_id)

@rpc("any_peer")
func _remote_stop() -> void:
	received_remote_stop.emit()

func _send_tick_data(peer_id: int, data: PackedByteArray) -> void:
	_rtd.rpc_id(peer_id, data)

func _is_network_host() -> bool:
	return multiplayer.is_server()

func _is_network_authority_for_node(node: Node) -> bool: return node.is_multiplayer_authority()

func _get_unique_id() -> int:
	return multiplayer.get_unique_id()

# _rit is short for _receive_tick_data. The method name ends up in each message
# so, we're trying to keep it short.
@rpc("any_peer", "unreliable")
func _rtd(data: PackedByteArray) -> void:
	received_tick_data.emit(multiplayer.get_remote_sender_id(), data, get_tree().root)
