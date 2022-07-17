use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{thread_rng, Rng};
use std::time::Duration;

use crate::{
    health::{Damaged, Health, Invuln},
    player::{Laser, Player},
    GameState,
};

const ENEMY_ONE_HEALTH: u8 = 5;
const ENEMY_ONE_COOLDOWN_SECS: f32 = 4.0;
const ENEMY_ONE_COOLDOWN_VAR: f32 = 1.0;
const ENEMY_ONE_SPEED: f32 = 70.0;
const ENEMY_ONE_BEAM_MS: u64 = 600;

pub struct EnemyPlugin;

struct DicethulhuSheet(Handle<TextureAtlas>);
struct EnemyOneSheet(Handle<TextureAtlas>);
struct EnemyOneBeamSprite(Handle<Image>);
//struct DiceRollSheet(Handle<Image>)

#[derive(Component)]
pub struct FacePlayer;
#[derive(Component, Debug)]
pub struct DiesToLaser;

#[derive(Component)]
pub struct Dicethulhu;

#[derive(Component)]
pub struct EnemyOne {
    state: EnemyOneState,
    attack_cooldown: Timer,
}
#[derive(PartialEq)]
enum EnemyOneState {
    Idle,
    Move,
    Attack,
}
#[derive(Component, Debug)]
pub struct Beam {
    timer: Timer,
}

impl Default for Beam {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(ENEMY_ONE_BEAM_MS), false),
        }
    }
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_dicethulhu)
            .add_system(animate_dicethulhu)
            .add_system_set(SystemSet::on_enter(GameState::Play).with_system(spawn_enemy_one))
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(face_player)
                    .with_system(check_for_laser)
                    .with_system(animate_enemy_one)
                    .with_system(destroy_beam)
                    .with_system(enemy_one_movement),
            );
    }
}

fn destroy_beam(mut commands: Commands, mut query: Query<(Entity, &mut Beam)>, time: Res<Time>) {
    for (id, mut beam) in query.iter_mut() {
        beam.timer.tick(time.delta());
        if beam.timer.just_finished() {
            commands.entity(id).despawn_recursive();
        }
    }
}

fn check_for_laser(
    mut commands: Commands,
    query: Query<(Entity, &DiesToLaser), Without<Invuln>>,
    laser_query: Query<&Laser>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for collision in collisions.iter() {
        if let CollisionEvent::Started(a, b, _) = collision {
            if query.get(*a).is_ok() && laser_query.get(*b).is_ok() {
                commands.entity(*a).insert(Damaged);
            } else if query.get(*b).is_ok() && laser_query.get(*a).is_ok() {
                commands.entity(*b).insert(Damaged);
            }
        }
    }
}

fn face_player(
    mut query: Query<(&FacePlayer, &mut TextureAtlasSprite, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single();
    for (_, mut sprite, transform) in query.iter_mut() {
        sprite.flip_x =
            match (player_transform.translation.x - transform.translation.x).signum() as i32 {
                1 => true,
                -1 => false,
                _ => unreachable!(),
            }
    }
}

fn animate_dicethulhu(
    mut query: Query<&mut TextureAtlasSprite, With<Dicethulhu>>,
    time: Res<Time>,
) {
    let mut dicethulhu_sprite = query.single_mut();
    dicethulhu_sprite.index = ((time.time_since_startup().as_millis() / 200) % 16)
        .try_into()
        .expect("Should always fit into u128");
}

fn spawn_dicethulhu(mut commands: Commands, sprite_sheet: Res<DicethulhuSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 40.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Dicethulhu)
        .insert(Name::from("Dicethulhu"));
}

fn enemy_one_movement(
    mut query: Query<(&mut Transform, &mut EnemyOne)>,
    player_query: Query<&Transform, (With<Player>, Without<EnemyOne>)>,
    time: Res<Time>,
) {
    let player_transform = player_query.single();
    for (mut enemy_transform, mut enemy_one) in query.iter_mut() {
        if enemy_one.state != EnemyOneState::Move {
            continue;
        }

        let y_delta = player_transform.translation.y - enemy_transform.translation.y;
        if y_delta.abs() < 1.0 {
            enemy_one.state = EnemyOneState::Idle;
        }
        let y_direction = y_delta.signum();
        enemy_transform.translation.y += ENEMY_ONE_SPEED * y_direction * time.delta_seconds();
    }
}

fn animate_enemy_one(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut TextureAtlasSprite,
        &mut Transform,
        &mut EnemyOne,
    )>,
    time: Res<Time>,
    beam_texture: Res<EnemyOneBeamSprite>,
) {
    let frame = (time.time_since_startup().as_millis() / 100) % 7;

    for (id, mut enemy_sprite, mut enemy_transform, mut enemy_one) in query.iter_mut() {
        if enemy_one.state == EnemyOneState::Attack && frame % 7 == 6 {
            enemy_one.state = EnemyOneState::Move;

            let mut rng = thread_rng();
            enemy_one.attack_cooldown = Timer::from_seconds(
                ENEMY_ONE_COOLDOWN_SECS
                    + rng.gen_range(-ENEMY_ONE_COOLDOWN_VAR..=ENEMY_ONE_COOLDOWN_VAR),
                false,
            );
        }

        enemy_one.attack_cooldown.tick(time.delta());
        if enemy_one.attack_cooldown.finished()
            && enemy_one.state == EnemyOneState::Idle
            && frame == 0
        {
            enemy_one.state = EnemyOneState::Attack;
        }

        let frame = match enemy_one.state {
            EnemyOneState::Idle | EnemyOneState::Move => (frame + 1) % 7,
            EnemyOneState::Attack => (frame % 7 + 7).min(11),
        };

        enemy_sprite.index = frame.try_into().expect("Should always fit into u128");

        const TOTAL_DISPLACEMENT: f32 = 1.0;

        let y_vel = match enemy_one.state {
            EnemyOneState::Idle | EnemyOneState::Move => {
                if frame >= 2 {
                    -TOTAL_DISPLACEMENT / 5.0
                } else {
                    TOTAL_DISPLACEMENT / 2.0
                }
            }
            EnemyOneState::Attack => 0.0,
        };

        if enemy_one.state == EnemyOneState::Attack && frame == 10 {
            let beam = commands
                .spawn_bundle(SpriteBundle {
                    texture: beam_texture.0.clone(),
                    transform: Transform {
                        translation: Vec3::new(
                            175.0
                                * match enemy_sprite.flip_x {
                                    true => 1.0,
                                    false => -1.0,
                                },
                            1.5,
                            20.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Collider::cuboid(167.0, 3.0))
                .insert(Sensor)
                .insert(Beam::default())
                .insert(Name::from("Beam"))
                .id();
            commands.entity(id).add_child(beam);
        }

        enemy_transform.translation.y += y_vel;
    }
}

fn spawn_enemy_one(mut commands: Commands, sprite_sheet: Res<EnemyOneSheet>) {
    let mut rng = thread_rng();
    for _ in 0..5 {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sprite_sheet.0.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        rng.gen_range(-150.0..=150.0),
                        rng.gen_range(-60.0..=60.0),
                        100.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Collider::cuboid(10.5, 8.0))
            .insert(Sensor)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(FacePlayer)
            .insert(DiesToLaser)
            .insert(EnemyOne {
                state: EnemyOneState::Idle,
                attack_cooldown: Timer::from_seconds(
                    ENEMY_ONE_COOLDOWN_SECS
                        + rng.gen_range(-ENEMY_ONE_COOLDOWN_VAR..=ENEMY_ONE_COOLDOWN_VAR),
                    false,
                ),
            })
            .insert(Health {
                health: ENEMY_ONE_HEALTH,
            })
            .insert(Name::from("EnemyOne"));
    }
}

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Dicethulhu.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(320.0, 180.0),
        16,
        1,
        Vec2::splat(2.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(DicethulhuSheet(atlas_handle));

    let image = assets.load("Enemy1.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::new(21.0, 16.0), 7, 2, Vec2::splat(2.0));
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(EnemyOneSheet(atlas_handle));

    let image = assets.load("Enemy1Beam.png");
    commands.insert_resource(EnemyOneBeamSprite(image));
}
