use godot::prelude::*;
use godot::engine::Sprite2D;
use godot::engine::ISprite2D;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct FunnyPlayer {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>
}

#[godot_api]
impl ISprite2D for FunnyPlayer {
    fn init(base: Base<Sprite2D>) -> Self {
        godot_print!("Hello, world!"); // Prints to the Godot console
        
        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // In GDScript, this would be: 
        // rotation += angular_speed * delta
        
        let radians = (self.angular_speed * delta) as f32;
        self.base_mut().rotate(radians);
        // The 'rotate' method requires a f32, 
        // therefore we convert 'self.angular_speed * delta' which is a f64 to a f32
    }
}