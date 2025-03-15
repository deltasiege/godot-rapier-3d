extends Node

var cmd = preload("res://tests/test_tools/cmd.gd")

func _ready(): 
	var test = cmd.get_cmdline_args().get("test")
	if test: _load_test(test)
	else: _load_demo("characters/characters")

func _load_demo(demo_name: String): _load_scene("res://demos/" + demo_name + ".tscn")
func _load_test(test_name: String): _load_test("res://demos/" + test_name + ".tscn")

func _load_scene(path: String):
	print("Loading: ", path)
	var scene = load(path)
	var instance = scene.instantiate()
	add_child(instance)
