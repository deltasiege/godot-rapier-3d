class_name GR3DStateFrame

var tick: int
var state_hash: int

var peer_hashes := {}
var mismatch := false

func _init(_tick: int, _state_hash: int) -> void:
	tick = _tick
	state_hash = _state_hash

func record_peer_hash(peer_id: int, peer_hash: int) -> bool:
	peer_hashes[peer_id] = peer_hash
	if peer_hash != state_hash:
		mismatch = true
		return false
	return true

func has_peer_hash(peer_id: int) -> bool:
	return peer_hashes.has(peer_id)

func is_complete(peers: Dictionary) -> bool:
	for peer_id in peers:
		if not peer_hashes.has(peer_id):
			return false
	return true

func get_missing_peers(peers: Dictionary) -> Array:
	var missing := []
	for peer_id in peers:
		if not peer_hashes.has(peer_id):
			missing.append(peer_id)
	return missing
