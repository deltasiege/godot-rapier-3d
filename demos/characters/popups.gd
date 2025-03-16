extends Node

var _initial_snapshot
var _last_snapshot: PackedByteArray
var _last_snapshot_data = {}
var opened_popup = null

var Hash = preload("res://test_assets/tools/hash.gd")

func _ready():
	get_parent().toolbar.connect("popup_opened", on_popup_opened)
	await get_tree().physics_frame
	await get_tree().physics_frame
	_initial_snapshot = GR3D.get_snapshot()

func _process(_delta):
	if Input.is_action_just_pressed("toggle_sim"): toggle_sim()
	if Input.is_action_just_pressed("reset_sim"): reset_sim()
	if Input.is_action_pressed("step_sim_continuous") or Input.is_action_just_pressed("step_sim_single"): step()
	if Input.is_action_just_pressed("save_snapshot"): take_snapshot()
	if Input.is_action_just_pressed("load_snapshot"): restore_snapshot()

func _physics_process(_delta):
	if opened_popup: opened_popup.current_content.set_entries(_get_data(opened_popup.title))

func on_popup_opened(popup: Control):
	opened_popup = popup
	opened_popup.current_content.force_set_entries(_get_data(opened_popup.title))

func _get_data(title: String):
	var chr = get_parent()._active_character
	match title:
		"Character":
			var common_data = {
				"type": chr.get_class(),
				"velocity": chr.velocity.snappedf(0.1),
				"real_velocity": chr.get_real_velocity().snappedf(0.1),
				"real_angular_velocity": chr.get_real_angular_velocity().snappedf(0.1),
				"is_on_floor": chr.is_on_floor(),
			}
			match chr.get_class():
				"RapierKinematicCharacter3D":
					return common_data.merged({
						"last_motion": chr.get_last_motion().snappedf(0.1),
						"is_sliding_down_slope": chr.is_sliding_down_slope(),
						"slide_collision_count": chr.get_slide_collision_count(),
					})
				"RapierPIDCharacter3D":
					return common_data.merged({})
		"State":
			return [
				{ "text": "time", "value": snapped(GR3D.get_time(), 0.1) },
				{ "text": "tick", "value": GR3D.get_tick() },
				{ "type": "button", "id": "pause_sim", "text": "Pause simulation" if !GR3DRuntime.paused else "Play simulation", "on_pressed": toggle_sim },
				{ "type": "button", "text": "Reset simulation", "on_pressed": reset_sim  },
				{ "type": "button", "text": "Advance 1 tick", "on_pressed": step },
				{ "type": "button", "text": "Take snapshot", "on_pressed": take_snapshot },
				{ "type": "button", "text": "Restore snapshot", "on_pressed": restore_snapshot },
				{ "text": "godot_hash", "value": _last_snapshot_data.get("godot_hash") },
				{ "text": "rapier_hash", "value": _last_snapshot_data.get("rapier_hash") },
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
	_last_snapshot = GR3D.get_snapshot()
	_last_snapshot_data = {
		"snapshot_bytes": _last_snapshot.size(),
		"godot_hash": Hash.get_godot_hash(self),
		"rapier_hash": Array(_last_snapshot.compress()).hash()
	}
