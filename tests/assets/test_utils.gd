class_name TestUtils

static func _get_cmdline_args() -> Dictionary:
	var args = {}
	for arg in OS.get_cmdline_user_args():
		if arg.find("=") > -1:
			var key_value = arg.split("=")
			args[key_value[0].lstrip("--")] = key_value[1]
		else:
			args[arg.lstrip("--")] = ""
	return args
