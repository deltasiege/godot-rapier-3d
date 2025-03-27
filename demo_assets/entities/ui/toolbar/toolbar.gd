extends Control

@export var popup_titles: Array[String]
@export var button_icons: Array[CompressedTexture2D]
@export var popup_scene: PackedScene = preload("res://demo_assets/entities/ui/drag_panel/drag_panel.tscn")
@export var content_scene: PackedScene = preload("res://demo_assets/entities/ui/dictionary_display/dictionary_display.tscn")
@export var button_container: Control

@onready var popups_container: Control = $Panels

var open_popup
signal popup_opened(popup: Control)

func _ready():
	for idx in popup_titles.size(): create_button(popup_titles[idx], button_icons[idx])

func create_button(popup_title: String, icon: CompressedTexture2D):
	var button = Button.new()
	var tex = TextureRect.new()
	button.custom_minimum_size.y = 32
	button.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	button_container.add_child(button)
	button.add_child(tex)
	
	tex.texture = icon
	tex.expand_mode = TextureRect.EXPAND_FIT_WIDTH
	tex.custom_minimum_size = Vector2(24, 24)
	tex.set_anchors_and_offsets_preset(PRESET_FULL_RECT, PRESET_MODE_MINSIZE, 4)
	
	button.owner = button_container
	button.connect("pressed", func(): create_popup(popup_title))

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
