use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevity::BevityPlugin;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use bevy_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraPlugin, ThirdPersonCameraTarget, Zoom,
};
use bevy_tnua::prelude::*;
use exported::{BevityExported, DoRotate, PlayerCharacter};

mod exported;

fn main() {
    println!("starting");
    App::new()
        .add_plugins((DefaultPlugins, BevityPlugin::<BevityExported>::default()))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TnuaRapier3dPlugin)
        .add_plugins(TnuaControllerPlugin)
        .add_plugins(ThirdPersonCameraPlugin)
        .add_systems(Update, initialize_player)
        .add_systems(Update, initialize_camera)
        .add_systems(Update, player_control_system)
        .add_systems(Update, do_rotate)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .run();
}

fn initialize_player(
    query: Query<Entity, (Without<TnuaController>, With<PlayerCharacter>)>,
    mut commands: Commands,
) {
    for entity in &query {
        let mut cmd = commands.entity(entity);
        cmd.insert(TnuaRapier3dIOBundle::default());
        cmd.insert(TnuaControllerBundle::default());
        cmd.insert(ThirdPersonCameraTarget);
    }
}

fn do_rotate(mut query: Query<(&mut Transform, &DoRotate)>, time: Res<Time>) {
    for (mut transform, rot) in query.iter_mut() {
        transform.rotate_axis(
            Vec3::Y,
            rot.speed * std::f32::consts::TAU * time.delta_seconds(),
        );
    }
}

fn initialize_camera(
    query: Query<Entity, (Without<ThirdPersonCamera>, With<Camera>)>,
    mut commands: Commands,
) {
    for entity in &query {
        let mut cmd = commands.entity(entity);
        cmd.insert(ThirdPersonCamera {
            aim_enabled: false,
            cursor_lock_toggle_enabled: false,
            cursor_lock_active: false,
            mouse_sensitivity: 5.0,
            // mouse_orbit_button_enabled: true,
            // mouse_orbit_button: MouseButton::Left,
            zoom: Zoom::new(10.0, 10.0),
            ..default()
        });
        cmd.insert(Collider::ball(1.0));
        cmd.insert(RigidBody::KinematicPositionBased);
    }
}

fn player_control_system(
    mut query: Query<(&mut TnuaController, &PlayerCharacter)>,
    keys: Res<Input<KeyCode>>,
) {
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::D) {
        direction += Vec3::X;
    }

    if keys.pressed(KeyCode::A) {
        direction -= Vec3::X;
    }

    if keys.pressed(KeyCode::W) {
        direction -= Vec3::Z;
    }

    if keys.pressed(KeyCode::S) {
        direction += Vec3::Z;
    }

    let jumped = keys.pressed(KeyCode::Space);

    for (mut controller, pc) in &mut query {
        controller.basis(TnuaBuiltinWalk {
            // Move in the direction the player entered, at a speed of 10.0:
            desired_velocity: direction * pc.move_speed,

            // Turn the character in the movement direction:
            desired_forward: direction,

            // Must be larger than the height of the entity's center from the bottom of its
            // collider, or else the character will not float and Tnua will not work properly:
            float_height: pc.float_height,

            // TnuaBuiltinWalk has many other fields that can be configured:
            ..Default::default()
        });

        if jumped {
            // The jump action must be fed as long as the player holds the button.
            controller.action(TnuaBuiltinJump {
                // The full height of the jump, if the player does not release the button:
                height: pc.jump_height,
                shorten_extra_gravity: 0.0,
                // TnuaBuiltinJump too has other fields that can be configured:
                ..Default::default()
            });
        }
    }
}

// fn update_system(
//     mut controllers: Query<(
//         &Name,
//         &Transform,
//         &PlayerCharacter,
//         &GravityScale,
//         &mut KinematicCharacterController,
//     )>,
//     keys: Res<Input<KeyCode>>,
// ) {
//     for (name, transform, pc, g, mut controller) in &mut controllers {
//         let mut direction = -Vec3::Y * g.0;

//         if keys.pressed(KeyCode::D) {
//             direction += Vec3::X * pc.move_speed;
//         }

//         if keys.pressed(KeyCode::A) {
//             direction -= Vec3::X * pc.move_speed;
//         }

//         let position = transform.translation + direction;

//         controller.translation = Some(position);

//         tracing::info!(
//             "moving character: {} to {} from {}",
//             name,
//             position,
//             transform.translation
//         );
//     }
// }
