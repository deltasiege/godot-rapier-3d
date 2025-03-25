extends Node

const INITIAL_FRAME = 0
const MAX_ROLLBACK_FRAMES = 20 # How far back can we reload previous game states
const ADVANTAGE_LIMIT = 20 # How many ticks can any peer get ahead/behind another peer

var peers := {}
var players := {}

var started = false

var local_tick = INITIAL_FRAME
var peer_ticks := {} # { peer_id: peer's current tick }
var peer_advantages := {} # { peer_id: peer's advantage (how far ahead/behind) }
var peer_actions := {} # { peer_id: [ActionBufferStep, ..], .. }
var completed_tick = INITIAL_FRAME # Latest frame that all peers are action complete and state is synchronized

var Classes = preload("./classes.gd")
#var RPC = preload("./rpc.gd")

# store game state = in rust already
# Restore game state to the given tick (half in rust - lets see)

func _physics_process(_delta: float) -> void:
	if not started: return
	
	# TODO send input ticks
	# TODO Receive peer ticks and advantages
	# TODO receive peer serialized actions and place into peer_actions
	
	

func determine_peer_completed_tick(peer_id: int) -> int:
	if !peer_ticks.has(peer_id): return -1
	if !peer_actions.has(peer_id): return -1
	var peer_tick = peer_ticks[peer_id]
	var final_tick = local_tick if peer_tick > local_tick else peer_tick # Only check inputs up to the current tick of the slowest party
	var action_buffer = peer_actions[peer_id]
	# select frames from (sync_frame + 1) through final_frame and find the first frame where predicted and remote inputs don't match
	
	return -1

# Returns false if there is no need to rollback for the given peer because we are both complete already
func rollback_is_possible(peer_id: int) -> bool:
	if peer_ticks.has(peer_id): return false
	return local_tick > completed_tick and peer_ticks[peer_id] > completed_tick

# Returns true if the local client is acceptably time synced with the given peer
func time_is_synced(peer_id: int) -> bool:
	if peer_ticks.has(peer_id): return false
	if peer_advantages.has(peer_id): return false
	var local_advantage = local_tick - peer_ticks[peer_id]
	var advantage_delta = local_advantage - peer_advantages[peer_id]
	return local_advantage < MAX_ROLLBACK_FRAMES and advantage_delta <= ADVANTAGE_LIMIT
