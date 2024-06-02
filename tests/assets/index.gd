extends Node

func _ready(): 
	var test = TestUtils._get_cmdline_args().get("test")
	if !test: return
	_load_test(test)

func _load_test(test_name: String):
	print("Loading test: ", test_name)
	var scene = load("res://tests/" + test_name + ".tscn")
	var instance = scene.instantiate()
	add_child(instance)


