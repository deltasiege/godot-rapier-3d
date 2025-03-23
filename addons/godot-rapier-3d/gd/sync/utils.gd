static func get_system_time_msecs() -> int:
	return int(round(1000.0 * Time.get_unix_time_from_system()))
