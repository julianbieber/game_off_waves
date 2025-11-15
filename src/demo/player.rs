//! Player-specific behavior.

use avian2d::prelude::{AngularDamping, Collider, CollisionLayers, LinearDamping, Mass, RigidBody};
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    AppSystems, PausableSystems,
    demo::{
        GameCollisionLayer,
        movement::MovementController,
        weapons::{WeaponSlots, WeaponType},
    },
    screens::Screen,
};

#[derive(Component)]
pub struct PlayerStats {
    pub projectile_damage_percentage: f32,
    pub projectile_speed_percentage: f32,
    pub projectile_rate_percentage: f32,
    pub _explosion_damage_percentage: f32,
}
impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            projectile_damage_percentage: 1.0,
            projectile_speed_percentage: 1.0,
            projectile_rate_percentage: 1.0,
            _explosion_damage_percentage: 1.0,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            record_player_directional_input
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            follow_cam
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        ),
    )
    .add_systems(Update, update_time.run_if(in_state(Screen::Gameplay)))
    .add_plugins(Material2dPlugin::<BoatMaterial>::default());
}

/// The player character.
pub fn player(
    max_speed: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<BoatMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Rectangle::new(300.0, 500.0));
    let material = materials.add(BoatMaterial { time: Vec4::ZERO });

    let collision = CollisionLayers::new(
        GameCollisionLayer::Player,
        [GameCollisionLayer::Terrain, GameCollisionLayer::Enemy],
    );
    let mut stats = PlayerStats::default();
    stats.projectile_rate_percentage = 0.2;
    (
        Name::new("Player"),
        Player,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::ZERO),
        MovementController {
            max_speed,
            ..default()
        },
        RigidBody::Dynamic,
        Mass(10.0),
        AngularDamping(2.0),
        LinearDamping(0.2),
        Collider::rectangle(100.0, 200.0),
        collision,
        WeaponSlots {
            left: [
                Some(WeaponType::default_cannon(&stats)),
                Some(WeaponType::default_cannon(&stats)),
                Some(WeaponType::default_cannon(&stats)),
            ],
            right: [
                Some(WeaponType::default_cannon(&stats)),
                Some(WeaponType::default_cannon(&stats)),
                Some(WeaponType::default_cannon(&stats)),
            ],
            front: Some(WeaponType::default_cannon(&stats)),
            // front: None,
        },
        stats,
    )
}

fn follow_cam(
    camera: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera = camera.into_inner();

    let player = player.into_inner();

    camera.translation.x = player.translation.x;
    camera.translation.y = player.translation.y;

    // let cam_angle = camera.rotation.to_euler(EulerRot::XYZ).2;
    // let player_angle = player.rotation.to_euler(EulerRot::XYZ).2;
    // let diff = (cam_angle - player_angle).abs();

    // if diff < 0.01 {
    // camera.rotation = player.rotation;
    // } else {
    // camera.rotation = Quat::from_rotation_z(cam_angle - (cam_angle - player_angle) * 0.2);
    // }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = 0.0;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent -= 1.0;
    }

    let mut rotation_intent = 0.0;
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        rotation_intent += 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        rotation_intent -= 1.0;
    }
    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
        controller.rotation_intent = rotation_intent;
    }
}

fn update_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<BoatMaterial>>,
    boats: Query<&MeshMaterial2d<BoatMaterial>>,
) {
    for c in boats.iter() {
        if let Some(m) = materials.get_mut(c.0.id()) {
            m.time = Vec4::new(time.elapsed_secs(), 0.0, 0.0, 0.0);
        }
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BoatMaterial {
    #[uniform(0)]
    time: Vec4,
}

const BOAT_SHADER_PATH: &str = "shaders/boat.wesl";

impl Material2d for BoatMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        BOAT_SHADER_PATH.into()
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
