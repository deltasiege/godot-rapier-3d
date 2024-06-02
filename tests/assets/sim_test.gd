extends Node3D

@export var fps = 600
@export var total_steps = 500
@export var report_file_name = "report.txt"
@export var quit_on_complete = true
var step_counter = 0
var report = []
var report_rel_path
var report_abs_path
var report_generated := false

func _ready():
	Engine.set_physics_ticks_per_second(fps)
	print("Simulating: '", name, "' for ", total_steps, " steps at: " + str(Engine.physics_ticks_per_second) + " physics ticks per second")
	var dt = str(int(Time.get_unix_time_from_system() * 1000))
	var target = TestUtils._get_cmdline_args().get("target") if !OS.is_debug_build() else "debug";
	if !target and !OS.is_debug_build(): 
		push_error("Target not specified via commandline (e.g. --target=x86_64-pc-windows-msvc)")
		get_tree().root.propagate_notification(NOTIFICATION_WM_CLOSE_REQUEST)
		get_tree().call_deferred("quit")
		
	report_rel_path = "reports/" + target + "-" + dt + "-" + report_file_name
	if OS.is_debug_build():
		report_abs_path = OS.get_user_data_dir() + "/" + report_rel_path
	else:
		var prefix = ""
		if OS.get_name() == "macOS": prefix = OS.get_executable_path().get_base_dir().get_base_dir().get_base_dir().get_base_dir()
		else: prefix = OS.get_executable_path().get_base_dir()
		report_abs_path = prefix + "/" + report_rel_path

func _physics_process(_delta):
	if step_counter <= total_steps:
		step_counter += 1
		if step_counter % (total_steps / floor(5)) == 0: print("Step: " + str(step_counter))
		Rapier3D.step()
		var rapier_hash = Rapier3D.get_hash(Rapier3D.get_state())
		var godot_hash = Rapier3D.get_godot_hash(Rapier3D.get_godot_state())
		report.append({ "rapier_hash": rapier_hash, "godot_hash": godot_hash })
	else:
		if report_generated: return
		save_report(report, report_abs_path)
		print("Report saved to: '", report_abs_path, "'")
		report_generated = true
		if quit_on_complete:
			get_tree().root.propagate_notification(NOTIFICATION_WM_CLOSE_REQUEST)
			get_tree().quit()

func get_env_info():
	return [
		"OS.get_name: " + OS.get_name(),
		"OS.get_version: " + OS.get_version(),
		"OS.get_processor_count: " + str(OS.get_processor_count()),
		"OS.get_processor_name: " + OS.get_processor_name(),
		"OS.get_video_adapter_driver_info: " + ", ".join(OS.get_video_adapter_driver_info()),
		"Engine.get_architecture_name: " + Engine.get_architecture_name(),
		"Engine.physics_ticks_per_second: " + str(Engine.physics_ticks_per_second),
	]

func save_report(entries: Array, path: String):
	var file = FileAccess.open(path, FileAccess.WRITE)
	if file == null: 
		push_error("Could not access: ", path)
		return
	file.store_line("---")
	file.store_line("environment")
	file.store_line("---")
	for line in get_env_info():
		file.store_line(line)
	
	file.store_line("---")
	file.store_line("step#, rapier hash, godot hash")
	file.store_line("---")
	for idx in entries.size():
		var entry = report[idx]
		file.store_line(str(idx + 1) + ", " + str(entry.rapier_hash) + ", " + str(entry.godot_hash))
