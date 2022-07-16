use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

use crate::physics::GroundDetection;
use crate::PlayerSheet;

pub struct PlayerPlugin;

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

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(animate_player)
            .add_system(player_movement);
    }
}

fn spawn_player(mut commands: Commands, sprite_sheet: Res<PlayerSheet>) {
    let sprite = TextureAtlasSprite::new(0);

    commands
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
            speed: 100.0,
            jump_force: 200.0,
            anim_state: PlayerAnimState::Idle,
        })
        .insert(GroundDetection::default())
        .insert(Name::new("Player"));
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
    let up = keyboard.pressed(KeyCode::W)
        || keyboard.pressed(KeyCode::Up)
        || keyboard.pressed(KeyCode::Space);

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
