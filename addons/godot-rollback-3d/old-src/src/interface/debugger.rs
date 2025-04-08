use crate::World;
use godot::prelude::*;
use rapier3d::prelude::{
    DebugRenderBackend, DebugRenderObject, DebugRenderPipeline, DebugRenderStyle, ShapeType,
};

pub struct GR3DDebugger {
    pub backend: GR3DDebuggerBackend,
    pipeline: DebugRenderPipeline,
}

impl GR3DDebugger {
    pub fn new() -> Self {
        Self {
            backend: GR3DDebuggerBackend {
                pending_lines: Array::new(),
            },
            pipeline: DebugRenderPipeline::render_all(DebugRenderStyle::default()),
        }
    }

    pub fn render(&mut self, world: &World) -> Array<Array<Variant>> {
        self.pipeline.render(
            &mut self.backend,
            &world.physics.bodies,
            &world.physics.colliders,
            &world.physics.impulse_joints,
            &world.physics.multibody_joints,
            &world.physics.narrow_phase,
        );

        let result = self.backend.pending_lines.duplicate_shallow();
        self.backend.pending_lines.clear();
        return result;
    }
}

pub struct GR3DDebuggerBackend {
    pub pending_lines: Array<Array<Variant>>,
}

impl DebugRenderBackend for GR3DDebuggerBackend {
    fn draw_line(
        &mut self,
        _object: rapier3d::prelude::DebugRenderObject,
        a: rapier3d::prelude::Point<f32>,
        b: rapier3d::prelude::Point<f32>,
        color: rapier3d::prelude::DebugColor,
    ) {
        let mut entry = Array::new();
        entry.push(&Vector3::new(a.x, a.y, a.z).to_variant());
        entry.push(&Vector3::new(b.x, b.y, b.z).to_variant());
        entry.push(
            &Color::from_ok_hsl(color[0] as f64, color[1] as f64, color[2] as f64)
                .with_alpha(color[3])
                .to_variant(),
        );
        self.pending_lines.push(&entry);
    }

    fn filter_object(&self, object: DebugRenderObject) -> bool {
        match object {
            DebugRenderObject::Collider(_, collider) => {
                if let ShapeType::TriMesh = collider.shape().shape_type() {
                    let casted = collider.shape().as_trimesh();
                    if let Some(trimesh) = casted {
                        let too_many_verts =
                            trimesh.vertices().len() > crate::config::DEBUG_MAX_VERTEX_COUNT;
                        return !too_many_verts;
                    } else {
                        return true;
                    }
                }
            }
            _ => return true,
        }

        true
    }
}
