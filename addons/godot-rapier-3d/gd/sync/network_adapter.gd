extends Node
class_name GR3DNetAdapter

static func is_type(obj: Object):
	return obj.has_method("attach_network_adapter") \
		and obj.has_method("detach_network_adapter") \
		and obj.has_method("start_network_adapter") \
		and obj.has_method("stop_network_adapter") \
		and obj.has_method("send_ping") \
		and obj.has_method("send_ping_back") \
		and obj.has_method("send_remote_start") \
		and obj.has_method("send_remote_stop") \
		and obj.has_method("send_input_tick") \
		and obj.has_method("is_server") \
		and obj.has_method("is_authority_of_node") \
		and obj.has_method("get_unique_id") \
		and obj.has_method("poll")

signal received_ping(peer_id, msg)
signal received_ping_back(peer_id, msg)
signal received_remote_start()
signal received_remote_stop()
signal received_input_tick(peer_id, msg)

func attach_network_adapter(sync) -> void: pass
func detach_network_adapter(sync) -> void: pass
func start_network_adapter(sync) -> void: pass
func stop_network_adapter(sync) -> void: pass

func send_ping(peer_id: int, msg: Dictionary) -> void: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_ping()")
func send_ping_back(peer_id: int, msg: Dictionary) -> void: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_ping_back()")
func send_remote_start(peer_id: int) -> void: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_remote_start()")
func send_remote_stop(peer_id: int) -> void: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_remote_stop()")
func send_input_tick(peer_id: int, msg: PackedByteArray) -> void: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.send_input_tick()")
func is_server() -> bool: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.is_server()"); return true
func is_authority_of_node(node: Node) -> bool: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.is_authority_of_node()"); return true
func get_unique_id() -> int: push_error("UNIMPLEMENTED ERROR: NetworkAdapter.get_unique_id()"); return 1
func poll() -> void: pass
