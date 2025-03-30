extends Node

var cmd = preload("res://test_assets/tools/cmd.gd")
const DEFAULT_DEMO = "multiplayer/multiplayer"

func _ready():
	print("[MAIN]")
	var test_name = cmd.get_cmdline_args().get("test")
	var path = _get_test_path(test_name) if test_name else _get_demo_path(DEFAULT_DEMO)
	print("Loading: ", path)
	_load_scene(path)

func _get_demo_path(demo_name: String) -> String: return "res://demos/" + demo_name + ".tscn"
func _get_test_path(test_name: String) -> String: return "res://tests/" + test_name + ".tscn"

func _load_scene(path: String):
	var scene = load(path)
	var instance = scene.instantiate()
	add_child(instance)
