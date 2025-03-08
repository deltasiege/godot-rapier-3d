use godot::classes::IRefCounted;
use godot::classes::RefCounted;
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
                DebugRenderStyle {
                    subdivisions: 20,
                    border_subdivisions: 5,
                    collider_dynamic_color: [340.0, 1.0, 0.3, 1.0],
                    collider_kinematic_color: [131.0, 1.0, 0.3, 1.0],
                    collider_fixed_color: [30.0, 1.0, 0.4, 1.0],
                    collider_parentless_color: [30.0, 1.0, 0.4, 1.0],
                    impulse_joint_anchor_color: [240.0, 0.5, 0.4, 1.0],
                    impulse_joint_separation_color: [0.0, 0.5, 0.4, 1.0],
                    multibody_joint_anchor_color: [300.0, 1.0, 0.4, 1.0],
                    multibody_joint_separation_color: [0.0, 1.0, 0.4, 1.0],
                    sleep_color_multiplier: [1.0, 1.0, 0.2, 1.0],
                    disabled_color_multiplier: [0.0, 0.0, 1.0, 1.0],
                    rigid_body_axes_length: 0.5,
                    contact_depth_color: [120.0, 1.0, 0.4, 1.0],
                    contact_normal_color: [0.0, 1.0, 1.0, 1.0],
                    contact_normal_length: 0.3,
                    collider_aabb_color: [124.0, 1.0, 0.4, 1.0],
                },
                DebugRenderMode::all(),
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
    pub fn render(&mut self) {
        self.try_render().map_err(crate::handle_error).ok();
    }

    pub fn try_render(&mut self) -> Result<(), String> {
        let engine = crate::get_engine()?;
        let bind = engine.bind();
        self.debug_render_pipeline.render(
            &mut self.debug_render_backend,
            &bind.pipeline.state.rigid_body_set,
            &bind.pipeline.state.collider_set,
            &bind.pipeline.state.impulse_joint_set,
            &bind.pipeline.state.multibody_joint_set,
            &bind.pipeline.state.narrow_phase,
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
                        Color::from_ok_hsl(
                            (color[0] / 255.0) as f64,
                            color[1] as f64,
                            color[2] as f64,
                        )
                        .with_alpha(color[3]),
                    ),
                ];
                node.call("_draw_line", args);
            }
            None => {
                log::error!(
                    "RapierDebugRenderBackend::draw_line - Trying to draw_line but no debugger node registered"
                );
            }
        }
    }
}
