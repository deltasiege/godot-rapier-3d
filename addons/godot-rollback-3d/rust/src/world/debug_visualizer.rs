use godot::prelude::*;
use rapier3d::prelude::*;

use crate::world::PhysicsState;

pub struct DebugVisualizer {
    pub backend: DebugVisualizerBackend,
    pipeline: DebugRenderPipeline,
}

impl DebugVisualizer {
    pub fn new() -> Self {
        Self {
            backend: DebugVisualizerBackend {
                pending_lines: Array::new(),
            },
            pipeline: DebugRenderPipeline::render_all(DebugRenderStyle::default()),
        }
    }

    pub fn render(&mut self, physics: &PhysicsState) -> Array<Array<Variant>> {
        self.pipeline.render(
            &mut self.backend,
            &physics.bodies,
            &physics.colliders,
            &physics.impulse_joints,
            &physics.multibody_joints,
            &physics.narrow_phase,
        );

        let result = self.backend.pending_lines.duplicate_shallow();
        self.backend.pending_lines.clear();
        return result;
    }
}

pub struct DebugVisualizerBackend {
    pub pending_lines: Array<Array<Variant>>,
}

impl DebugRenderBackend for DebugVisualizerBackend {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        a: Point<f32>,
        b: Point<f32>,
        color: DebugColor,
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
                        let too_many_verts = trimesh.vertices().len()
                            > crate::config::DEBUG_MAX_VERTEX_COUNT as usize;
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
