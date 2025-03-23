extends RefCounted
class_name GR3DPeer

var _peer_id: int
var peer_id: int:
	get: return _peer_id
	set(v): pass

var _spectator: bool = false
var spectator: bool:
	get: return _spectator
	set(v): pass

var rtt: int
var last_ping_received: int
var time_delta: float

var last_remote_input_tick_received: int = 0
var next_local_input_tick_requested: int = 1
var last_remote_hash_tick_received: int = 0
var next_local_hash_tick_requested: int = 1

var remote_lag: int
var local_lag: int

var calculated_advantage: float
var advantage_list := []

func _init(p_peer_id: int, p_options: Dictionary) -> void:
	_peer_id = p_peer_id
	_spectator = p_options.get('spectator', false)

func record_advantage(ticks_to_calculate_advantage: int) -> void:
	advantage_list.append(local_lag - remote_lag)
	if advantage_list.size() >= ticks_to_calculate_advantage:
		var total: float = 0
		for x in advantage_list:
			total += x
		calculated_advantage = total / advantage_list.size()
		advantage_list.clear()

func clear_advantage() -> void:
	calculated_advantage = 0.0
	advantage_list.clear()

func clear() -> void:
	rtt = 0
	last_ping_received = 0
	time_delta = 0
	last_remote_input_tick_received = 0
	next_local_input_tick_requested = 0
	last_remote_hash_tick_received = 0
	next_local_hash_tick_requested = 0
	remote_lag = 0
	local_lag = 0
	clear_advantage()
