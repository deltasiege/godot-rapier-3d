use godot::engine::IRefCounted;
use godot::engine::RefCounted;
use godot::prelude::*;
use rapier3d::pipeline::*;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base = RefCounted)]
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
        self.debug_render_backend.debugger_node = Some(debugger_node);
    }

    #[func]
    pub fn render_colliders(&mut self) {
        self.try_render_colliders()
            .map_err(crate::handle_error)
            .ok();
    }

    pub fn try_render_colliders(&mut self) -> Result<(), String> {
        let engine = crate::get_engine()?;
        let bind = engine.bind();
        self.debug_render_pipeline.render_colliders(
            &mut self.debug_render_backend,
            &bind.pipeline.state.rigid_body_set,
            &bind.pipeline.state.collider_set,
        );
        Ok(())
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
                    Variant::from(Vector3::new(a.x as f32, a.y as f32, a.z as f32)),
                    Variant::from(Vector3::new(b.x as f32, b.y as f32, b.z as f32)),
                    Variant::from(
                        Color::from_hsv(color[0] as f64, color[1] as f64, color[2] as f64)
                            .with_alpha(color[3]),
                    ),
                ];
                node.call(StringName::from("_draw_line"), args);
            }
            None => {
                log::error!(
                    "RapierDebugRenderBackend::draw_line - Trying to draw_line but no debugger node registered"
                );
            }
        }
    }
}
