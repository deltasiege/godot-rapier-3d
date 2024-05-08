use crate::physics_state::PhysicsState;
use godot::engine::IRefCounted;
use godot::engine::RefCounted;
use godot::prelude::*;
use rapier3d::pipeline::*;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct RapierDebugRenderPipeline {
    debug_render_pipeline: DebugRenderPipeline,
    debug_render_backend: RapierDebugRenderBackend,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RapierDebugRenderPipeline {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            debug_render_pipeline: DebugRenderPipeline::new(
                DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES,
            ),
            debug_render_backend: RapierDebugRenderBackend::new(),
            base,
        }
    }
}

#[godot_api]
impl RapierDebugRenderPipeline {
    #[func]
    pub fn register_debugger(&mut self, debugger_node: Gd<Node3D>) {
        self.debug_render_backend.debugger_node = Some(debugger_node); //.cast::<Node3D>()
    }

    #[func]
    pub fn render_colliders(&mut self) {
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let state: &mut PhysicsState = &mut singleton.bind_mut().pipeline.state;
            let rigid_body_set: &RigidBodySet = &state.rigid_body_set;
            let collider_set: &ColliderSet = &state.collider_set;

            self.debug_render_pipeline.render_colliders(
                &mut self.debug_render_backend,
                rigid_body_set,
                collider_set,
            );
        } else {
            godot_error!("RapierDebugRenderPipeline::render_colliders - Could not access Rapier3DEngine singleton");
        }
    }
}

pub struct RapierDebugRenderBackend {
    debugger_node: Option<Gd<Node3D>>,
}

impl RapierDebugRenderBackend {
    pub fn new() -> Self {
        Self {
            debugger_node: None,
        }
    }
}

impl DebugRenderBackend for RapierDebugRenderBackend {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject<'_>,
        a: Point<Real>,
        b: Point<Real>,
        color: [f32; 4],
    ) {
        let debugger_node = &mut self.debugger_node;
        match debugger_node {
            Some(node) => {
                let args = &[
                    Variant::from(Vector2::new(a.x as f32, a.y as f32)),
                    Variant::from(Vector2::new(b.x as f32, b.y as f32)),
                    Variant::from(
                        Color::from_hsv(color[0] as f64, color[1] as f64, color[2] as f64)
                            .with_alpha(color[3]),
                    ),
                ];
                node.call(StringName::from("_draw_line"), args);
            }
            None => {
                godot_error!("RapierDebugRenderBackend::draw_line - Trying to draw_line but no debugger node registered");
            }
        }
    }
}
