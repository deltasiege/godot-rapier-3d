static func start(sync: GR3DSync) -> void:
	var net_adapter: GR3DNetAdapter = sync.network_adapter
	assert(net_adapter.is_server() or sync.mechanized, "start() should only be called on the host")
	if sync.started or sync.host_starting:
		return
	if sync.mechanized:
		sync.on_received_remote_start.call()
		return
	if net_adapter.is_server():
		var highest_rtt: int = 0
		for peer in sync.peers.values():
			highest_rtt = max(highest_rtt, peer.rtt)

		# Call _remote_start() on all the other peers.
		for peer_id in sync.peers:
			net_adapter.send_remote_start(peer_id)

		# Attempt to prevent double starting on the host.
		sync.host_starting = true

		# Wait for half the highest RTT to start locally.
		if not sync.mechanized:
			print ("Delaying host start by %sms" % (highest_rtt / 2))
			await sync.get_tree().create_timer(highest_rtt / 2000.0).timeout

		sync.on_received_remote_start.call()
		sync.host_starting = false

static func stop(sync: GR3DSync) -> void:
	var net_adapter: GR3DNetAdapter = sync.network_adapter
	if net_adapter.is_server() and not sync.mechanized:
		for peer_id in sync.peers:
			net_adapter.send_remote_stop(peer_id)
	sync.on_received_remote_stop.call()


static func reset(sync: GR3DSync) -> void:
	pass
	#sync.input_tick = 0
	#sync.current_tick = sync.input_tick - sync.input_delay
	#sync.skip_ticks = 0
	#sync.rollback_ticks = 0
	#sync.nput_buffer.clear()
	#sync.tate_buffer.clear()
	#sync.tate_hashes.clear()
	#sync.input_buffer_start_tick = 1
	#sync.state_buffer_start_tick = 0
	#sync.state_hashes_start_tick = 1
	#sync.input_send_queue.clear()
	#sync.input_send_queue_start_tick = 1
	#sync.ticks_spent_regaining_sync = 0
	#sync.interpolation_state.clear()
	#sync.time_since_last_tick = 0.0
	#sync.debug_skip_nth_message_counter = 0
	#sync.input_complete_tick = 0
	#sync.state_complete_tick = 0
	#sync.last_state_hashed_tick = 0
	#sync.state_mismatch_count = 0
	#sync.in_rollback = false
	#sync.ran_physics_process = false
	#sync.ticks_since_last_interpolation_frame = 0
