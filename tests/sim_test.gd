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
	var dt = Time.get_datetime_string_from_system(true).replace(":", "_")
	report_rel_path = "reports/" + dt + "-" + OS.get_name() + "-" + report_file_name
	if OS.is_debug_build():
		report_abs_path = OS.get_user_data_dir() + "/" + report_rel_path
	else:
		report_abs_path = OS.get_executable_path().get_base_dir() + "/" + report_rel_path

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
			get_tree().call_deferred("quit")

func get_env_info():
	return [
		"get_name: " + OS.get_name(),
		"get_version: " + OS.get_version(),
		"get_processor_count: " + str(OS.get_processor_count()),
		"get_processor_name: " + OS.get_processor_name(),
		"get_video_adapter_driver_info: " + ", ".join(OS.get_video_adapter_driver_info()),
		"fps: " + str(Engine.physics_ticks_per_second),
	]

func save_report(entries: Array, path: String):
	var file = FileAccess.open(path, FileAccess.WRITE)
	
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
