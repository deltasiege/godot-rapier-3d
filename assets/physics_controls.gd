extends CanvasLayer

var play = true
var initial_snapshot
var snapshot
var snapshot_hash

@onready var hash_label = $Panel/MarginContainer/GridContainer/Hash

func _ready():
	Rapier3D.physics_ready.connect(_on_physics_ready)

func _on_physics_ready():
	initial_snapshot = _save()

func _physics_process(_delta):
	if play: Rapier3D.step()

func _save() -> PackedByteArray:
	var snap = Rapier3D.get_state()
	snapshot_hash = Rapier3D.get_hash(snap)
	hash_label.text = "Hash: " + str(snapshot_hash)
	print("Saved snapshot: ", snapshot_hash)
	return snap

func _load(snap):
	Rapier3D.set_state(snap)
	print("Loaded snapshot: ", snapshot_hash)

func _on_reset_pressed(): _load(initial_snapshot)
func _on_step_button_pressed(): Rapier3D.step()
func _on_save_button_pressed(): snapshot = _save()
func _on_load_button_pressed(): _load(snapshot)
func _on_check_button_toggled(toggled_on): play = toggled_on
