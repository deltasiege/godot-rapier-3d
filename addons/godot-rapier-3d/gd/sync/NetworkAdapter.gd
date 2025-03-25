extends Node

static func is_type(obj: Object):
	return obj.has_method("on_attached") \
		and obj.has_method("on_detached") \
		and obj.has_method("send_ping") \
		and obj.has_method("send_ping_back") \
		and obj.has_method("send_remote_start") \
		and obj.has_method("send_remote_stop") \
		and obj.has_method("send_tick_data") \
		and obj.has_method("is_network_host") \
		and obj.has_method("is_network_authority_for_node") \
		and obj.has_method("get_unique_id")

signal received_ping(peer_id, msg)
signal received_ping_back(peer_id, msg)
signal received_remote_start()
signal received_remote_stop()
signal received_tick_data(peer_id, msg)

func on_attached() -> void: pass
func on_detached() -> void: pass

func send_ping(peer_id: int, msg: Dictionary) -> void:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_ping()")

func send_ping_back(peer_id: int, msg: Dictionary) -> void:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_ping_back()")

func send_remote_start(peer_id: int) -> void:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_remote_start()")

func send_remote_stop(peer_id: int) -> void:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_remote_stop()")

func send_tick_data(peer_id: int, msg: PackedByteArray) -> void:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_tick_data()")

func is_network_host() -> bool:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.is_network_host()")
	return true

func is_network_authority_for_node(node: Node) -> bool:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.is_network_authority_for_node()")
	return true

func get_unique_id() -> int:
	push_error("UNIMPLEMENTED ERROR: NetworkAdapter.get_unique_id()")
	return 1
