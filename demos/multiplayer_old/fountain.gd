extends MultiplayerSpawner

@export var primitives: Array[PackedScene]
var counter = 0
var _last_spawn_frame

func _physics_process(_delta):
	if GR3D.get_tick() % 100 == 0: spawn_primitive()

func spawn_primitive():
	if _last_spawn_frame == GR3D.get_tick(): return
	var idx = counter % primitives.size()
	var new_prim = primitives[idx].instantiate()
	new_prim.name = "Primitive " + str(counter)
	get_parent().add_child(new_prim, true)
	new_prim.owner = get_parent()
	_last_spawn_frame = GR3D.get_tick()
