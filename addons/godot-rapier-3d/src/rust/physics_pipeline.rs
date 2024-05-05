use crate::collider::RapierCollider3D;
use crate::rigid_body::RapierRigidBody3D;
use crate::utils::*;
use godot::engine::IRefCounted;
use godot::engine::RefCounted;
use godot::prelude::*;
use rapier3d::math::Vector as RVector;
use rapier3d::na;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct RapierPhysicsPipeline {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: na::SVector<f32, 3>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RapierPhysicsPipeline {
    fn init(base: Base<RefCounted>) -> Self {
        godot_print!("RapierPhysicsPipeline::init()");

        // let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
        // collider_set.insert(collider);

        // let rigid_body = RigidBodyBuilder::dynamic()
        //     .translation(vector![0.0, 10.0, 0.0])
        //     .build();
        // let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        // let ball_body_handle = rigid_body_set.insert(rigid_body);
        // collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

        Self {
            rigid_body_nodes: Vec::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: (),
            base,
        }
    }
}

#[godot_api]
impl RapierPhysicsPipeline {
    #[signal]
    fn rapier3d_physics_step();

    #[func]
    fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );

        self.base_mut()
            .emit_signal("rapier3d_physics_step".into(), &[]);
    }

    #[func]
    fn get_active_dynamic_body_ids(&self) -> Array<VariantArray> {
        self.island_manager
            .active_dynamic_bodies()
            .into_iter()
            .map(|handle| rb_handle_to_id(*handle))
            .collect()
    }

    #[func]
    fn sync_active_body_godot_transforms(&self) {
        // self.island_manager
        //     .active_dynamic_bodies()
        //     .iter()
        //     .for_each(|handle| {
        //         let id = rb_handle_to_id(*handle);
        //         let node: Gd<RapierRigidBody3D> = self
        //             .rigid_body_nodes
        //             .iter()
        //             .find(|node| node.bind().id == id)
        //             .unwrap();
        //         // let rigid_body = node.bind().rigid_body.clone();
        //         // let translation: RVector<Real> = rigid_body.translation().clone();
        //         // godot_print!("rb: {:?}", rigid_body);
        //         // let mut node_clone = node.clone();
        //         // let mut node_class = node_clone.bind_mut();
        //         // node_class.set_godot_position(translation); // TODO so much cloning... surely im doing something wrong
        //     })
    }

    #[func]
    fn add_rigid_body(&mut self, rigid_body: Gd<RapierRigidBody3D>) {
        rigid_body.bind_mut().join_set(&mut self.rigid_body_set); // TODO I don't want to clone - is it necessary to ask the rigid_body to join the set here?
    }

    #[func]
    fn remove_rigid_body(&mut self, mut rigid_body: Gd<RapierRigidBody3D>) {
        let mut rb = rigid_body.bind_mut();
        self.rigid_body_set.remove(
            rb.handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            false,
        );
        rb.handle = RigidBodyHandle::invalid();
        rb.id = Array::new();
    }

    #[func]
    fn count_rigid_bodies(&self) -> i64 {
        self.rigid_body_set.len() as i64
    }

    #[func]
    fn print_rigid_body_transforms(&mut self) {
        self.rigid_body_set.iter().for_each(|(handle, rigid_body)| {
            godot_print!(
                "{:?}: translation: {:?}, rotation: {:?}",
                handle,
                rigid_body.translation(),
                rigid_body.rotation()
            );
        });
    }

    #[func]
    fn add_collider_with_parent(
        &mut self,
        mut collider: Gd<RapierCollider3D>,
        parent: Gd<RapierRigidBody3D>,
    ) {
        let mut col = collider.bind_mut();
        let rb = parent.bind();
        let handle = self.collider_set.insert_with_parent(
            col.collider.clone(),
            rb.handle,
            &mut self.rigid_body_set,
        );
        col.handle = handle;
        col.parent = Some(rb.handle);
        let (index, generation) = handle.into_raw_parts();
        let mut id = Array::new();
        id.push(index);
        id.push(generation);
        col.id = id;
    }

    #[func]
    fn remove_collider(&mut self, mut collider: Gd<RapierCollider3D>) {
        let mut col = collider.bind_mut();
        self.collider_set.remove(
            col.handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            false,
        );
        col.handle = ColliderHandle::invalid();
        col.parent = None;
        col.id = Array::new();
    }

    #[func]
    fn count_colliders(&self) -> i64 {
        self.collider_set.len() as i64
    }
}
