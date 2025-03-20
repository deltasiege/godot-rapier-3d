extends Node

@export var kinematic_character: RapierKinematicCharacter3D
@export var pid_character: RapierPIDCharacter3D

@onready var _chars = [kinematic_character, pid_character]
var _active_char_idx = -1
var _active_character = null

func _ready():
	_switch_char()

func _process(_delta):
	if Input.is_action_just_pressed("switch_character") and is_piloting(): _switch_char()
	if Input.is_action_just_pressed("toggle_mouse"): 
		WindowMgr.toggle_mouse_captured()
		if !is_piloting(): _active_character.cam_pivot.process_mode = PROCESS_MODE_DISABLED
		else: _active_character.cam_pivot.process_mode = PROCESS_MODE_INHERIT

func _switch_char():
	var idx = _active_char_idx % _chars.size()
	var current_char = _chars[idx]
	current_char.process_mode = Node.PROCESS_MODE_DISABLED
	current_char.name = "Inactive Character"
	
	_active_char_idx += 1
	idx = _active_char_idx % _chars.size()
	_active_character = _chars[idx]
	_active_character.process_mode = Node.PROCESS_MODE_INHERIT
	_active_character.name = "Active Character"
	var cam: Camera3D = _active_character.find_child("Camera3D", true, false)
	cam.make_current()

func is_piloting():
	return Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
