extends SubViewportContainer

var initial_snapshot
var saved_snapshot

@onready var rapier_hash_label = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/RapierHash
@onready var godot_hash_label = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/GodotHash
@onready var play_button = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/PlayButton
@onready var canvas = $SubViewport/CanvasLayer

var Hash = preload("res://tests/test_tools/hash.gd")

func _ready():
	await get_tree().physics_frame
	await get_tree().physics_frame
	initial_snapshot = _save()
	play_button.set_pressed(GR3DRuntime.autoplay)

func _save() -> PackedByteArray:
	var snapshot = GR3D.get_snapshot()
	var rapier_hash = Hash.get_rapier_hash()
	var godot_hash = Hash.get_godot_hash(self)
	rapier_hash_label.text = "Rapier hash: " + str(rapier_hash)
	godot_hash_label.text = "Godot hash: " + str(godot_hash)
	return snapshot

func _load(snapshot):
	GR3D.restore_snapshot(snapshot)

func _toggle(enabled: bool):
	canvas.visible = enabled

func _process(_delta):
	if Input.is_action_just_pressed("toggle_sim"): GR3DRuntime.toggle_pause(!GR3DRuntime.paused)
	if Input.is_action_just_pressed("reset_sim"): _load(initial_snapshot)
	if Input.is_action_pressed("step_sim_continuous") or Input.is_action_just_pressed("step_sim_single"): GR3D.step(1)
	if Input.is_action_just_pressed("save_snapshot"): saved_snapshot = _save()
	if Input.is_action_just_pressed("load_snapshot"): _load(saved_snapshot)

func _on_play_button_toggled(toggled_on): GR3DRuntime.toggle_pause(!toggled_on)
func _on_reset_pressed(): _load(initial_snapshot)
func _on_step_button_pressed(): GR3D.step(1)
func _on_save_button_pressed(): saved_snapshot = _save()
func _on_load_button_pressed(): _load(saved_snapshot)
