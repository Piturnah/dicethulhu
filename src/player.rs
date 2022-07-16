use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

use crate::physics::GroundDetection;
use crate::{BulletSprite, GunSheet, PlayerSheet};

pub struct PlayerPlugin;

const PLAYER_SPEED: f32 = 100.0;
const GUN_COOLDOWN_MS: u64 = 100;
const GUN_TRAVEL_SPEED: f32 = 300.0;

#[derive(Inspectable)]
enum PlayerAnimState {
    Idle,
    Run,
    Jump,
}

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
    jump_force: f32,
    anim_state: PlayerAnimState,
}

#[derive(Component)]
pub struct Gun {
    timer: Timer,
}

#[derive(Component)]
pub struct GunNozzle;

#[derive(Component)]
pub struct Laser {
    direction: Dir,
}

enum Dir {
    Left,
    Right,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(animate_player)
            .add_system(player_movement)
            .add_system(gun_position)
            .add_system(shoot_gun)
            .add_system(bullet_travel);
    }
}

fn bullet_travel(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Laser)>,
    time: Res<Time>,
) {
    for (id, mut transform, laser) in query.iter_mut() {
        transform.translation.x += time.delta_seconds()
            * match laser.direction {
                Dir::Left => -GUN_TRAVEL_SPEED,
                Dir::Right => GUN_TRAVEL_SPEED,
            };

        if transform.translation.x.abs() > 180.0 {
            commands.entity(id).despawn_recursive();
        }
    }
}

fn shoot_gun(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut Gun, &mut TextureAtlasSprite, &GlobalTransform)>,
    time: Res<Time>,
    laser_sprite: Res<BulletSprite>,
) {
    let (mut gun, mut gun_sprite, gun_transform) = query.single_mut();

    gun_sprite.index = 0;

    gun.timer.tick(time.delta());

    if gun.timer.just_finished() && keyboard.pressed(KeyCode::Space) {
        gun_sprite.index = 1;

        commands
            .spawn_bundle(SpriteBundle {
                texture: laser_sprite.0.clone(),
                transform: Transform {
                    translation: gun_transform.translation + Vec3::new(0.0, 1.0, -1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Laser {
                direction: match gun_sprite.flip_x {
                    true => Dir::Left,
                    false => Dir::Right,
                },
            });
    }
}

fn gun_position(
    mut gun_query: Query<(&mut Transform, &mut TextureAtlasSprite), With<Gun>>,
    player_query: Query<&TextureAtlasSprite, (With<Player>, Without<Gun>)>,
) {
    let (mut gun_transform, mut gun_sprite) = gun_query.single_mut();
    let player_sprite = player_query.single();

    if player_sprite.flip_x {
        gun_transform.translation.x = -7.5;
    } else {
        gun_transform.translation.x = 7.5;
    }

    gun_sprite.flip_x = player_sprite.flip_x;
}

fn spawn_player(mut commands: Commands, sprite_sheet: Res<PlayerSheet>, gun_sheet: Res<GunSheet>) {
    let sprite = TextureAtlasSprite::new(0);

    let player = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(GravityScale::default())
        .insert(Collider::cuboid(8.0, 10.5))
        .insert(Player {
            speed: PLAYER_SPEED,
            jump_force: 200.0,
            anim_state: PlayerAnimState::Idle,
        })
        .insert(GroundDetection::default())
        .insert(Name::new("Player"))
        .id();

    let gun = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: gun_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(7.5, -4.5, 10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Gun {
            timer: Timer::new(Duration::from_millis(GUN_COOLDOWN_MS), true),
        })
        .insert(Name::from("Gun"))
        .id();

    let gun_nozzle = commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform {
            translation: Vec3::new(5.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GunNozzle)
        .insert(Name::from("Gun Nozzle"))
        .id();

    commands.entity(gun).add_child(gun_nozzle);
    commands.entity(player).add_child(gun);
}

fn player_movement(
    mut player_query: Query<(
        &mut Player,
        &mut Transform,
        &mut TextureAtlasSprite,
        &mut Velocity,
        &GroundDetection,
        &mut GravityScale,
    )>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform, mut sprite, mut vel, ground_sensor, mut gravity) =
        player_query.single_mut();

    let right = keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right);
    let left = keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left);
    let up = keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up);

    let mut delta_x = 0.0;
    if right {
        delta_x += player.speed * time.delta_seconds();
    }
    if left {
        delta_x -= player.speed * time.delta_seconds();
    }

    if delta_x != 0.0 {
        player.anim_state = PlayerAnimState::Run;

        sprite.flip_x = delta_x < 0.0;
    } else {
        player.anim_state = PlayerAnimState::Idle;
    }

    if !ground_sensor.grounded {
        player.anim_state = PlayerAnimState::Jump;

        if vel.linvel[1] < 0.0 {
            *gravity = GravityScale(1.5);
        }
    } else if up {
        vel.linvel = Vec2::new(0.0, player.jump_force);
    } else {
        *gravity = GravityScale(1.0);
    }

    transform.translation += Vec3::new(delta_x, 0.0, 0.0);
}

fn animate_player(
    mut player_query: Query<(&Player, &mut TextureAtlasSprite, &Velocity)>,
    time: Res<Time>,
) {
    let (player, mut sprite, velocity) = player_query.single_mut();

    sprite.index = match player.anim_state {
        PlayerAnimState::Idle => 0,
        PlayerAnimState::Run => ((time.time_since_startup().as_millis() / 100) % 5 + 1)
            .try_into()
            .expect("Spritesheet index should always fit into usize!"),
        PlayerAnimState::Jump => {
            if velocity.linvel[1] >= 0.0 {
                6
            } else {
                7
            }
        }
    };
}
