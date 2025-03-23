class_name GR3DActionFrame

var tick: int
var players := {}

func _init(_tick: int) -> void:
	tick = _tick

func get_player_input(peer_id: int) -> Dictionary:
	if players.has(peer_id):
		return players[peer_id].input
	return {}

func is_player_input_predicted(peer_id: int) -> bool:
	if players.has(peer_id):
		return players[peer_id].predicted
	return true

func get_missing_peers(peers: Dictionary) -> Array:
	var missing := []
	for peer_id in peers:
		if not players.has(peer_id) or players[peer_id].predicted:
			missing.append(peer_id)
	return missing

func is_complete(peers: Dictionary) -> bool:
	for peer_id in peers:
		if not players.has(peer_id) or players[peer_id].predicted:
			return false
	return true
