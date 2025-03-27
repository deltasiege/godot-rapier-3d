extends Node

var _initial_snapshot
var _last_snapshot: PackedByteArray
var _last_snapshot_data = {}
var opened_popup = null
var character

var Hash = preload("res://test_assets/tools/hash.gd")

func _ready():
	get_parent().connect("popup_opened", on_popup_opened)
	await get_tree().physics_frame
	await get_tree().physics_frame
	_initial_snapshot = GR3D.save_snapshot() # TODO - causes issue with duplicates

func _process(_delta):
	if Input.is_action_just_pressed("toggle_sim"): toggle_sim()
	if Input.is_action_just_pressed("reset_sim"): reset_sim()
	if Input.is_action_pressed("step_sim_continuous") or Input.is_action_just_pressed("step_sim_single"): step()
	if Input.is_action_just_pressed("save_snapshot"): take_snapshot()
	if Input.is_action_just_pressed("load_snapshot"): restore_snapshot()

func _physics_process(_delta):
	if opened_popup: opened_popup.current_content.set_entries(_get_data(opened_popup.title))

func on_popup_opened(popup: Control):
	character = get_tree().root.find_child("Active Character", true, false)
	opened_popup = popup
	opened_popup.current_content.force_set_entries(_get_data(opened_popup.title))

func _get_data(title: String):
	if title == "Character" and !character: return
	match title:
		"Network":
			var peer_data = GR3DSync.get_all_peer_data()
			var grouped = {}
			grouped["Local ID"] = str(multiplayer.get_unique_id())
			for peer in peer_data:
				var data = peer.duplicate()
				data.erase("peer_id")
				grouped["Peer " + str(peer.peer_id)] = data
			return grouped
		"Character":
			var common_data = {
				"type": character.get_class(),
				"velocity": character.velocity.snappedf(0.1),
				"real_velocity": character.get_real_velocity().snappedf(0.1),
				"real_angular_velocity": character.get_real_angular_velocity().snappedf(0.1),
				"is_on_floor": character.is_on_floor(),
			}
			match character.get_class():
				"RapierKinematicCharacter3D":
					return common_data.merged({
						"last_motion": character.get_last_motion().snappedf(0.1),
						"is_sliding_down_slope": character.is_sliding_down_slope(),
						"slide_collision_count": character.get_slide_collision_count(),
					})
				"RapierPIDCharacter3D":
					return common_data.merged({})
		"Playback":
			return [
				{ "key": "time", "value": snapped(GR3D.get_time(), 0.1) },
				{ "key": "tick", "value": GR3D.get_tick() },
				{ "type": "button", "id": "pause_sim", "key": "Pause simulation" if !GR3DRuntime.paused else "Play simulation", "on_pressed": toggle_sim },
				{ "type": "button", "key": "Advance 1 tick", "on_pressed": step },
			]
		"Snapshots":
			return [
				{ "type": "button", "key": "Reset simulation", "on_pressed": reset_sim  },
				{ "type": "button", "key": "Take snapshot", "on_pressed": take_snapshot },
				{ "type": "button", "key": "Restore snapshot", "on_pressed": restore_snapshot },
				{ "key": "snapshot_bytes", "value": _last_snapshot_data.get("snapshot_bytes") },
				{ "key": "godot_hash", "value": _last_snapshot_data.get("godot_hash") },
				{ "key": "rapier_hash", "value": _last_snapshot_data.get("rapier_hash") },
			]
		"Rollback":
			return [
				{ "key": "TBA", "value": "Show full buffer here" },
			]
		"Hotkeys":
			return {
				"Move character": "WASD",
				"Activate mouse": "Right mouse button",
				"Switch character": "F",
				"Pause/unpause": "V",
				"Reset": "/",
				"Step once": "[",
				"Step continually (hold)": "]",
				"Take snapshot": "R",
				"Restore snapshot": "T",
			}
		"World":
			return GR3D.get_counts()

func toggle_sim(): GR3DRuntime.toggle_pause(!GR3DRuntime.paused)
func reset_sim(): GR3D.restore_snapshot(_initial_snapshot)
func step(): GR3D.step(1)
func restore_snapshot(): GR3D.restore_snapshot(_last_snapshot)
func take_snapshot():
	_last_snapshot = GR3D.save_snapshot()
	_last_snapshot_data = {
		"snapshot_bytes": _last_snapshot.size(),
		"godot_hash": Hash.get_godot_hash(get_tree().root),
		"rapier_hash": Array(_last_snapshot.compress()).hash()
	}
