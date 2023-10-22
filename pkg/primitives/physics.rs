use crate::{FileReference, UnityQuaternion, UnityVector3};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityBoxCollider {
    #[serde(default, rename = "m_IsTrigger")]
    pub is_trigger: i32,

    #[serde(default, rename = "m_Size")]
    pub size: UnityVector3,

    #[serde(default, rename = "m_Center")]
    pub center: UnityVector3,
}

impl UnityBoxCollider {
    pub fn add_box_collider(&self, transform: &Transform, commands: &mut EntityCommands) {
        let scale_x = transform.scale.x;
        let scale_y = transform.scale.y;
        let scale_z = transform.scale.z;
        commands.insert(Collider::cuboid(
            0.5 * scale_x,
            0.5 * scale_y,
            0.5 * scale_z,
        ));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnitySphereCollider {
    #[serde(default, rename = "m_IsTrigger")]
    pub is_trigger: i32,

    #[serde(default, rename = "m_Radius")]
    pub radius: f32,

    #[serde(default, rename = "m_Center")]
    pub center: UnityVector3,
}

impl UnitySphereCollider {
    pub fn add_sphere_collider(&self, transform: &Transform, commands: &mut EntityCommands) {
        commands.insert(Collider::ball(transform.scale.x * 0.5));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityCapsuleCollider {
    #[serde(default, rename = "m_IsTrigger")]
    pub is_trigger: i32,

    #[serde(default, rename = "m_Radius")]
    pub radius: f32,

    #[serde(default, rename = "m_Height")]
    pub height: f32,

    #[serde(default, rename = "m_Center")]
    pub center: UnityVector3,

    #[serde(default, rename = "m_Direction")]
    pub direction: i32,
}

impl UnityCapsuleCollider {
    pub fn add_capsule_collider(&self, transform: &Transform, commands: &mut EntityCommands) {
        let radius_scale = transform.scale.x.max(transform.scale.z);
        let height_scale = transform.scale.y;

        commands.insert(Collider::capsule_y(
            self.height / 4.0 * height_scale,
            self.radius * radius_scale,
        )); // default y axis collider
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityMeshCollider {
    #[serde(default, rename = "m_IsTrigger")]
    pub is_trigger: i32,

    #[serde(default, rename = "m_Mesh")]
    pub mesh: FileReference,
}

#[derive(Component)]
pub struct NeedsMeshCollider;

impl UnityMeshCollider {
    pub fn add_mesh_collider(&self, commands: &mut EntityCommands) {
        commands.insert(NeedsMeshCollider);
    }
}

pub fn load_mesh_collider_system(
    query: Query<(Entity, &Handle<Mesh>), With<NeedsMeshCollider>>,
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh) in &query {
        let mut cmd = commands.entity(entity);
        let mesh = meshes.get(mesh).unwrap();
        let collider = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap();
        cmd.insert(collider);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityRigidbody {
    #[serde(default, rename = "m_Mass")]
    pub mass: f32,
    #[serde(default, rename = "m_Drag")]
    pub drag: f32,
    #[serde(default, rename = "m_Angular_Drag")]
    pub angular_drag: f32,
    #[serde(default, rename = "m_CenterOfMass")]
    pub center_of_mass: UnityVector3,
    #[serde(default, rename = "m_InertiaTensor")]
    pub inertia_tensor: UnityVector3,
    #[serde(default, rename = "m_InertiaRotation")]
    pub inertia_rotation: UnityQuaternion,
    #[serde(default, rename = "m_UseGravity")]
    pub use_gravity: i32,
    #[serde(default, rename = "m_IsKinematic")]
    pub is_kinematic: i32,
    #[serde(default, rename = "m_Contraints")]
    pub constraints: i32,
}

impl UnityRigidbody {
    pub fn add_rigidbody(&self, commands: &mut EntityCommands) {
        if self.is_kinematic == 1 {
            commands.insert(Sensor);

            return;
        }

        let local_center_of_mass: Vec3 = self.center_of_mass.into();
        let principal_inertia_local_frame = self.inertia_rotation.into();
        let principal_inertia = self.inertia_tensor.into();

        commands.insert((
            RigidBody::Dynamic,
            AdditionalMassProperties::MassProperties(MassProperties {
                local_center_of_mass,
                mass: self.mass,
                principal_inertia_local_frame,
                principal_inertia,
            }),
            Damping {
                linear_damping: self.drag,
                angular_damping: self.angular_drag,
            },
        ));

        if self.use_gravity == 1 {
            commands.insert(GravityScale(1.0));
        } else {
            commands.insert(GravityScale(0.0));
        }
    }
}
