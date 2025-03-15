extends Control

@export var popup_scene: PackedScene = preload("res://entities/ui/drag_panel/drag_panel.tscn")
@export var content_scene: PackedScene = preload("res://entities/ui/dictionary_display/dictionary_display.tscn")

@onready var popups_container: Control = $Panels

var open_popup
signal popup_opened(popup: Control)

func create_popup(title: String):
	if open_popup != null:
		open_popup.close()
		if open_popup.title == title: return
	var new_popup = popup_scene.instantiate()
	new_popup.title = title
	popups_container.add_child(new_popup)
	new_popup.owner = popups_container
	
	var new_content = content_scene.instantiate()
	new_popup.append_content(new_content)
	
	open_popup = new_popup
	emit_signal("popup_opened", new_popup)

func _on_character_pressed(): create_popup("Character")
func _on_state_pressed(): create_popup("State")
func _on_hotkeys_pressed(): create_popup("Hotkeys")
func _on_world_pressed(): create_popup("World")
