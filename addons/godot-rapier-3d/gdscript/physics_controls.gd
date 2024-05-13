extends SubViewportContainer

var play = false
var should_show = false
var initial_snapshot
var snapshot
var rapier_hash
var godot_hash

@onready var rapier_hash_label = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/RapierHash
@onready var godot_hash_label = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/GodotHash
@onready var play_button = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/PlayButton
@onready var canvas = $SubViewport/CanvasLayer

func _ready():
	_update_should_show()
	if !should_show: return
	Rapier3D.physics_ready.connect(_on_physics_ready)
	if !Engine.is_editor_hint(): 
		play_button.set_pressed(true)
		play = true

func _on_physics_ready():
	initial_snapshot = _save()

func _physics_process(_delta):
	if !should_show: return
	if play: Rapier3D.step()

func _save() -> PackedByteArray:
	var snap = Rapier3D.get_state()
	var godot_snap = Rapier3D.get_godot_state()
	rapier_hash = Rapier3D.get_hash(snap)
	godot_hash = Rapier3D.get_godot_hash(godot_snap)
	rapier_hash_label.text = "Rapier hash: " + str(rapier_hash)
	godot_hash_label.text = "Godot hash: " + str(godot_hash)
	return snap

func _load(snap):
	Rapier3D.set_state(snap)

func _on_settings_changed(): _update_should_show()

func _update_should_show():
	var run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
	var run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
	var show_ui = ProjectSettings.get_setting("debug/rapier_3d/show_ui")
	if Engine.is_editor_hint() and !run_in_editor: should_show = false
	elif !Engine.is_editor_hint() and !run_in_game: should_show = false
	elif !show_ui: should_show = false
	else: should_show = true
	canvas.visible = should_show

func _on_reset_pressed(): _load(initial_snapshot)
func _on_step_button_pressed(): Rapier3D.step()
func _on_save_button_pressed(): snapshot = _save()
func _on_load_button_pressed(): _load(snapshot)
func _on_play_button_toggled(toggled_on): play = toggled_on

func _enter_tree(): ProjectSettings.connect("settings_changed", self._on_settings_changed)
func _exit_tree(): ProjectSettings.disconnect("settings_changed", self._on_settings_changed)
