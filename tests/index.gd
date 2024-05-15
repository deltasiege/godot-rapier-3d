extends Node

func _ready():
	var args = _process_args()
	var test = args.get("test")
	if !test: return
	_load_test(test)

func _load_test(test_name: String):
	print("Loading test: ", test_name)
	var scene = load("res://tests/" + test_name + ".tscn")
	var instance = scene.instantiate()
	add_child(instance)

func _process_args():
	var args = {}
	for arg in OS.get_cmdline_user_args():
		if arg.find("=") > -1:
			var key_value = arg.split("=")
			args[key_value[0].lstrip("--")] = key_value[1]
		else:
			args[arg.lstrip("--")] = ""
	return args
