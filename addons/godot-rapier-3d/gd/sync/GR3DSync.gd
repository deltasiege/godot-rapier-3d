extends Node

var peers := {}
var player_peers := {}
var network_adapter: GR3DNetAdapter
var message_serializer = preload("./message_serializer.gd")

var mechanized := false
var started := false
var host_starting := false
var spectating := false

var tick_time: float
var ping_frequency: float
var input_tick: int
var current_tick: int
var skip_ticks: int
var rollback_ticks: int
var requested_input_complete_tick: int

var max_buffer_size := 20
var ticks_to_calculate_advantage := 60
var max_action_frames_per_message := 5
var max_messages_at_once := 2
var max_ticks_to_regain_sync := 300
var min_lag_to_regain_sync := 5
var interpolation := false
var max_state_mismatch_count := 10

var action_buffer := []
var action_buffer_start_tick: int
var action_send_queue := []
var action_send_queue_start_tick: int
var state_hashes := []
var state_hashes_start_tick: int

signal sync_started()
signal sync_stopped()
signal sync_lost()
signal sync_regained()
signal sync_error(msg)

signal skip_ticks_flagged(count)
signal rollback_flagged(tick)
signal prediction_missed(tick, peer_id, local_input, remote_input)
signal remote_state_mismatch(tick, peer_id, local_hash, remote_hash)

signal peer_added(peer_id)
signal peer_removed(peer_id)
signal peer_pinged_back(peer)

signal state_loaded(_rollback_ticks)
signal tick_finished(is_rollback)
signal tick_retired(tick)
signal tick_input_complete(tick)
signal scene_spawned(name, spawned_node, scene, data)
signal scene_despawned(name, node)
signal interpolation_frame()

var on_received_ping: Callable
var on_received_ping_back: Callable
var on_received_remote_start: Callable
var on_received_remote_stop: Callable
var on_received_input_tick: Callable

var Actions = preload("./actions.gd")
var Peers = preload("./peers.gd")
var Lifecycle = preload("./lifecycle.gd")
var NetworkCallbacks = preload("./network_callbacks.gd")
var RPCAdapter = preload("./rpc_adapter.gd")
var Utils = preload("./utils.gd")

func _ready(): 
	NetworkCallbacks.init(self, RPCAdapter.new())

func start(): Lifecycle.start(self)
func stop(): Lifecycle.stop(self)
func reset(): Lifecycle.reset(self)

func clear_peers(): Peers.clear_peers(self)
func add_peer(peer_id: int): Peers.add_peer(self, peer_id)
func remove_peer(peer_id: int): Peers.remove_peer(self, peer_id)



func _physics_process(_delta: float) -> void:
	if not started: return
	
	var local_actions = GR3D._get_serialized_actions() # [timestep_id, num_actions, hash, bytes]
	if local_actions.size() == 0: return
	
	var timestep_id = local_actions[0]
	var ser_local_actions = local_actions[3]
	
	var ab_frame = Actions.get_or_create_ab_frame(self, timestep_id)
	ab_frame.players[network_adapter.get_unique_id()] = ser_local_actions # TODO predicted?
	
	if peers.size() <= 0: return # Only send actions when we have real remote peers
	action_send_queue.append(ser_local_actions)
	Peers.send_action_messages_to_all_peers(self)
