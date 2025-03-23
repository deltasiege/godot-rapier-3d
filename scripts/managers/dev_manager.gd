extends Node

@export var SPLIT_SCREEN_STYLE := VERTICAL
@export var USE_RATIO := true

func _ready():
	var screen_rect = DisplayServer.screen_get_usable_rect()
	var screen_ratio = screen_rect.size.aspect()
	if SPLIT_SCREEN_STYLE == HORIZONTAL:
		get_window().size.x = screen_rect.size.x / 2
		get_window().size.y = get_window().size.x / screen_ratio if USE_RATIO else screen_rect.size.y
		get_window().position.y = screen_rect.position.y
	else:
		get_window().size.y = screen_rect.size.y / 2
		get_window().size.x = get_window().size.y * screen_ratio if USE_RATIO else screen_rect.size.x
		get_window().position.x = screen_rect.position.x

	if "--server" in OS.get_cmdline_args():
		get_window().position = screen_rect.position
	elif "--client" in OS.get_cmdline_args():
		if SPLIT_SCREEN_STYLE == HORIZONTAL:
			get_window().position.x = screen_rect.size.x / 2
		else:
			get_window().position.y = screen_rect.size.y / 2
