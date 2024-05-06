# Godot Rapier 3D

## Contributing

See [CONTRIBUTING.md]()

## Credits

Inspired by [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

## Flow

1. singleton loaded
1. RRB class enters tree
1. RRB call singleton register function - store reference to the class to lookup later for changing node position via base_mut() + add to set -> return handle + id to RRB
1. GDscript calls step() in singleton

1. step() singleton island manager gets active IRB in set (returns array of IRB handles)
1. singelton looks up stored classes - check the handles of each one and compare
1. when matched, singleton gets ref to base using base_mut()
1. singleton sets base_mut() transform to IRB transform

1. when GRB moved in editor, GRB calls singleton providing handle to get RRB instance and then modifies it
