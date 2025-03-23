static func init(sync: GR3DSync, network_adapter: GR3DNetAdapter):
	sync.on_received_ping = on_received_ping(sync)
	sync.on_received_ping_back = on_received_ping_back(sync)
	sync.on_received_remote_start = on_received_remote_start(sync)
	sync.on_received_remote_stop = on_received_remote_stop(sync)
	sync.on_received_input_tick = on_received_input_tick(sync)
	set_network_adapter(sync, network_adapter)

static func set_network_adapter(sync: GR3DSync, new_adapter: Object) -> void:
	assert(GR3DNetAdapter.is_type(new_adapter), "Network adaptor is missing a some methods")
	assert(not sync.started, "Changing the network adapter after GR3DSync has started will probably break everything")

	if sync.network_adapter != null:
		sync.network_adapter.detach_network_adapter(sync)
		sync.network_adapter.received_ping.disconnect(sync.on_received_ping)
		sync.network_adapter.received_ping_back.disconnect(sync.on_received_ping_back)
		sync.network_adapter.received_remote_start.disconnect(sync.on_received_remote_start)
		sync.network_adapter.received_remote_stop.disconnect(sync.on_received_remote_stop)
		sync.network_adapter.received_input_tick.disconnect(sync.on_received_input_tick)

		sync.remove_child(sync.network_adapter)
		sync.network_adapter.queue_free()

	sync.network_adapter = new_adapter
	sync.network_adapter.name = 'NetworkAdaptor'
	sync.add_child(sync.network_adapter)
	sync.network_adapter.received_ping.connect(sync.on_received_ping)
	sync.network_adapter.received_ping_back.connect(sync.on_received_ping_back)
	sync.network_adapter.received_remote_start.connect(sync.on_received_remote_start)
	sync.network_adapter.received_remote_stop.connect(sync.on_received_remote_stop)
	sync.network_adapter.received_input_tick.connect(sync.on_received_input_tick)
	sync.network_adapter.attach_network_adapter(sync)

static func on_ping_timer_timeout(sync: GR3DSync) -> Callable:
	return func():
		if sync.peers.size() == 0: return
		var msg = { local_time = sync._get_system_time_msecs() }
		for peer_id in sync.peers:
			assert(peer_id != sync.network_adapter.get_unique_id(), "Cannot ping ourselves")
			sync.network_adapter.send_ping(peer_id, msg)

static func on_received_ping(sync: GR3DSync) -> Callable:
	return func(peer_id: int, msg: Dictionary):
		assert(peer_id != sync.network_adapter.get_unique_id(), "Cannot ping back ourselves")
		msg['remote_time'] = sync.get_system_time_msecs()
		sync.network_adapter.send_ping_back(peer_id, msg)

static func on_received_ping_back(sync: GR3DSync) -> Callable:
	return func(peer_id: int, msg: Dictionary):
		var system_time = sync.get_system_time_msecs()
		var peer = sync.peers[peer_id]
		peer.last_ping_received = system_time
		peer.rtt = system_time - msg['local_time']
		peer.time_delta = msg['remote_time'] - msg['local_time'] - (peer.rtt / 2.0)
		sync.peer_pinged_back.emit(peer)

static func on_received_remote_start(sync: GR3DSync) -> Callable:
	return func():
		sync.reset()
		#sync.tick_time = (1.0 / Engine.physics_ticks_per_second) # TODO set rapier Integration param DT
		sync.started = true
		sync.network_adapter.start_network_adapter(sync)
		#sync.spawn_manager.reset() # TODO
		sync.sync_started.emit()

static func on_received_remote_stop(sync: GR3DSync) -> Callable:
	return func():
		if not (sync.started or sync.host_starting): return

		sync.network_adapter.stop_network_adapter(sync)
		sync.started = false
		sync.host_starting = false
		sync.reset()

		for peer in sync.peers.values(): peer.clear()

		sync.sync_stopped.emit()
		#_spawn_manager.reset()
		sync.spectating = false

static func on_received_input_tick(sync: GR3DSync) -> Callable:
	return func(peer_id: int, serialized_msg: PackedByteArray):
		pass
		#if not sync.started: return
		#var msg = sync.message_serializer.unserialize_message(serialized_msg)
		#var peer: GR3DPeer = sync.peers[peer_id]
		## Record the next frame the other peer needs.
		#peer.next_local_input_tick_requested = max(msg[MessageSerializer.InputMessageKey.NEXT_INPUT_TICK_REQUESTED], peer.next_local_input_tick_requested)
		## Record the next state hash that the other peer needs.
		#peer.next_local_hash_tick_requested = max(msg[MessageSerializer.InputMessageKey.NEXT_HASH_TICK_REQUESTED], peer.next_local_hash_tick_requested)
#
		#var all_remote_input: Dictionary = msg[MessageSerializer.InputMessageKey.INPUT]
		#if all_remote_input.size() == 0:
			#return
#
		#var all_remote_ticks = all_remote_input.keys()
		#all_remote_ticks.sort()
#
		#var first_remote_tick = all_remote_ticks[0]
		#var last_remote_tick = all_remote_ticks[-1]
#
		#if first_remote_tick >= _input_tick + max_buffer_size:
			## This either happens because we are really far behind (but maybe, just
			## maybe could catch up) or we are receiving old ticks from a previous
			## round that hadn't yet arrived. Just discard the message and hope for
			## the best, but if we can't keep up, another one of the fail safes will
			## detect that we are out of sync.
			#print ("Discarding message from the future")
			## We return because we don't even want to do the accounting that happens
			## after integrating input, since the data in this message could be
			## totally bunk (ie. if it's from a previous match).
			#return
#
		#if _logger:
			#_logger.begin_interframe()
#
		## Only process if it contains ticks we haven't received yet.
		#if last_remote_tick > peer.last_remote_input_tick_received:
			## Integrate the input we received into the input buffer.
			#for remote_tick in all_remote_ticks:
				## Skip ticks we already have.
				#if remote_tick <= peer.last_remote_input_tick_received:
					#continue
				## This means the input frame has already been retired, which can only
				## happen if we already had all the input.
				#if remote_tick < _input_buffer_start_tick:
					#continue
#
				#var remote_input = _message_serializer.unserialize_input(all_remote_input[remote_tick])
#
				#var input_frame := _get_or_create_input_frame(remote_tick)
				#if input_frame == null:
					## _get_or_create_input_frame() will have already flagged the error,
					## so we can just return here.
					#return
#
				## If we already have non-predicted input for this peer, then skip it.
				#if not input_frame.is_player_input_predicted(peer_id):
					#continue
#
				##print ("Received remote tick %s from %s" % [remote_tick, peer_id])
				#if _logger:
					#_logger.add_value('remote_ticks_received_from_%s' % peer_id, remote_tick)
#
				## If we received a tick in the past and we aren't already setup to
				## rollback earlier than that...
				#var tick_delta = _current_tick - remote_tick
				#if tick_delta >= 0 and _rollback_ticks <= tick_delta:
					## Grab our predicted input, and store the remote input.
					#var local_input = input_frame.get_player_input(peer_id)
					#input_frame.players[peer_id] = InputForPlayer.new(remote_input, false)
#
					## Check if the remote input matches what we had predicted, if not,
					## flag that we need to rollback.
					#if local_input['$'] != remote_input['$']:
						#_rollback_ticks = tick_delta + 1
						#prediction_missed.emit(remote_tick, peer_id, local_input, remote_input)
						#rollback_flagged.emit(remote_tick)
				#else:
					## Otherwise, just store it.
					#input_frame.players[peer_id] = InputForPlayer.new(remote_input, false)
#
			## Find what the last remote tick we received was after filling these in.
			#var index = (peer.last_remote_input_tick_received - _input_buffer_start_tick) + 1
			#while index < input_buffer.size() and not input_buffer[index].is_player_input_predicted(peer_id):
				#peer.last_remote_input_tick_received += 1
				#index += 1
#
			## Update _input_complete_tick for new input.
			#_update_input_complete_tick()
#
		## Number of frames the remote is predicting for us.
		#if not _spectating:
			#peer.remote_lag = (peer.last_remote_input_tick_received + 1) - peer.next_local_input_tick_requested
#
		## Process state hashes.
		#var remote_state_hashes = msg[MessageSerializer.InputMessageKey.STATE_HASHES]
		#for remote_tick in remote_state_hashes:
			#var state_hash_frame := _get_state_hash_frame(remote_tick)
			#if state_hash_frame and not state_hash_frame.has_peer_hash(peer_id):
				#if not state_hash_frame.record_peer_hash(peer_id, remote_state_hashes[remote_tick]):
					#remote_state_mismatch.emit(remote_tick, peer_id, state_hash_frame.state_hash, remote_state_hashes[remote_tick])
#
		## Find what the last remote state hash we received was after filling these in.
		#var index = (peer.last_remote_hash_tick_received - _state_hashes_start_tick) + 1
		#while index < state_hashes.size() and state_hashes[index].has_peer_hash(peer_id):
			#peer.last_remote_hash_tick_received += 1
			#index += 1
