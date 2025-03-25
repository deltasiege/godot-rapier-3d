extends Timer

func _ready():
	name = "PingTimer"
	wait_time = 1.0
	autostart = true
	one_shot = false
	process_mode = Node.PROCESS_MODE_ALWAYS
	connect("timeout", GR3DSync._on_ping_timer_timeout)
