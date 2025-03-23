static func add_peer(sync: GR3DSync, peer_id: int, options: Dictionary = {}) -> void:
	assert(not sync.peers.has(peer_id), "Peer with given id already exists")
	assert(peer_id != sync.network_adapter.get_unique_id(), "Cannot add ourselves as a peer")

	if sync.peers.has(peer_id):
		return
	if peer_id == sync.network_adapter.get_unique_id():
		return

	_add_peer(sync, peer_id, options)
	sync.peer_added.emit(peer_id)

static func _add_peer(sync: GR3DSync, peer_id: int, options: Dictionary) -> void:
	var peer = GR3DPeer.new(peer_id, options)
	sync.peers[peer_id] = peer
	if not peer.spectator:
		sync.player_peers[peer_id] = peer

static func has_peer(sync: GR3DSync, peer_id: int) -> bool:
	return sync.peers.has(peer_id)

static func get_peer(sync: GR3DSync, peer_id: int) -> GR3DPeer:
	return sync.peers.get(peer_id)

static func get_player_peer_ids(sync: GR3DSync) -> Array:
	return sync.player_peers.keys()

static func get_player_peer_count(sync: GR3DSync) -> int:
	return sync.player_peers.size()

static func remove_peer(sync: GR3DSync, peer_id: int) -> void:
	if sync.peers.has(peer_id):
		_remove_peer(sync, peer_id)
		sync.peer_removed.emit(peer_id)
	if sync.peers.size() == 0: sync.stop()

static func _remove_peer(sync: GR3DSync, peer_id: int) -> void:
	sync.peers.erase(peer_id)
	if sync.player_peers.has(peer_id):
		sync.player_peers.erase(peer_id)

static func update_peer(sync: GR3DSync, peer_id: int, options: Dictionary = {}) -> void:
	assert(sync.peers.has(peer_id), "No peer with given id already exists")

	if sync.peers.has(peer_id):
		_remove_peer(sync, peer_id)
		_add_peer(sync, peer_id, options)

static func clear_peers(sync: GR3DSync) -> void:
	for peer_id in sync.peers.keys().duplicate():
		remove_peer(sync, peer_id)

static func get_state_hashes_for_peer(sync: GR3DSync, peer: GR3DPeer) -> Dictionary:
	var ret := {}
	if peer.next_local_hash_tick_requested >= sync.state_hashes_start_tick:
		var index = peer.next_local_hash_tick_requested - sync.state_hashes_start_tick
		while index < sync.state_hashes.size():
			var state_hash_frame: GR3DStateFrame = sync.state_hashes[index]
			ret[state_hash_frame.tick] = state_hash_frame.state_hash
			index += 1
	return ret

static func get_action_messages_from_send_queue_for_peer(sync: GR3DSync, peer: GR3DPeer) -> Array:
	var first_index := peer.next_local_input_tick_requested - sync.action_send_queue_start_tick
	var last_index := sync.action_send_queue.size() - 1
	var max_messages: int = (sync.max_action_frames_per_message * sync.max_messages_at_once)

	if (last_index + 1) - first_index <= max_messages:
		return get_action_messages_from_send_queue_in_range(sync, first_index, last_index, true)

	var new_messages = int(ceil(sync.max_messages_at_once / 2.0))
	var old_messages = int(floor(sync.max_messages_at_once / 2.0))

	return get_action_messages_from_send_queue_in_range(sync, last_index - (new_messages * sync.max_action_frames_per_message) + 1, last_index, true) + \
		   get_action_messages_from_send_queue_in_range(sync, first_index, first_index + (old_messages * sync.max_action_frames_per_message) - 1)

static func get_action_messages_from_send_queue_in_range(sync: GR3DSync, first_index: int, last_index: int, reverse: bool = false) -> Array:
	var indexes = range(first_index, last_index + 1) if not reverse else range(last_index, first_index - 1, -1)

	var all_messages := []
	var msg := {}
	for index in indexes:
		msg[sync.action_send_queue_start_tick + index] = sync.action_send_queue[index]

		if sync.max_action_frames_per_message > 0 and msg.size() == sync.max_action_frames_per_message:
			all_messages.append(msg)
			msg = {}

	if msg.size() > 0:
		all_messages.append(msg)

	return all_messages

static func send_action_messages_to_all_peers(sync: GR3DSync) -> void:
	for peer_id in sync.peers: send_action_messages_to_peer(sync, peer_id)

static var MessageSerializer = preload("./message_serializer.gd")

static func send_action_messages_to_peer(sync: GR3DSync, peer_id: int) -> void:
	assert(peer_id != sync.network_adapter.get_unique_id(), "Cannot send input to ourselves")
	var peer = sync.peers[peer_id]

	var state_hashes = get_state_hashes_for_peer(sync, peer)
	var input_messages = get_action_messages_from_send_queue_for_peer(sync, peer)

	for input in input_messages:
		var msg = {
			MessageSerializer.InputMessageKey.NEXT_INPUT_TICK_REQUESTED: peer.last_remote_input_tick_received + 1,
			MessageSerializer.InputMessageKey.INPUT: input,
			MessageSerializer.InputMessageKey.NEXT_HASH_TICK_REQUESTED: peer.last_remote_hash_tick_received + 1,
			MessageSerializer.InputMessageKey.STATE_HASHES: state_hashes,
		}

		var bytes = sync.message_serializer.serialize_message(msg) # TODO watch bytes.size()
		sync.network_adapter.send_input_tick(peer_id, bytes)
