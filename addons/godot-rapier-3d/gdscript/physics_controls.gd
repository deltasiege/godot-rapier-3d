extends SubViewportContainer

var play = false
var initial_snapshot
var snapshot
var snapshot_hash

@onready var hash_label = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/Hash
@onready var play_button = $SubViewport/CanvasLayer/Panel/MarginContainer/GridContainer/PlayButton
@onready var canvas = $SubViewport/CanvasLayer

func _ready():
	Rapier3D.physics_ready.connect(_on_physics_ready)
	if !Engine.is_editor_hint(): 
		play_button.set_pressed(true)
		play = true

func _on_physics_ready():
	initial_snapshot = _save()

func _physics_process(_delta):
	if play: Rapier3D.step()

func _save() -> PackedByteArray:
	var snap = Rapier3D.get_state()
	snapshot_hash = Rapier3D.get_hash(snap)
	hash_label.text = "Hash: " + str(snapshot_hash)
	return snap

func _load(snap):
	Rapier3D.set_state(snap)

func _on_settings_changed():
	var should_show = true
	var run_in_game = ProjectSettings.get_setting("debug/rapier_3d/debug_in_game")
	var run_in_editor = ProjectSettings.get_setting("debug/rapier_3d/debug_in_editor")
	canvas.visible = ProjectSettings.get_setting("debug/rapier_3d/show_ui")
	if Engine.is_editor_hint() and !run_in_editor: should_show = false
	elif !Engine.is_editor_hint() and !run_in_game: should_show = false

func _on_reset_pressed(): _load(initial_snapshot)
func _on_step_button_pressed(): Rapier3D.step()
func _on_save_button_pressed(): snapshot = _save()
func _on_load_button_pressed(): _load(snapshot)
func _on_play_button_toggled(toggled_on): play = toggled_on

func _enter_tree(): ProjectSettings.connect("settings_changed", self._on_settings_changed)
func _exit_tree(): ProjectSettings.disconnect("settings_changed", self._on_settings_changed)
