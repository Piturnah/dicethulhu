use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

use crate::player::{Laser, Player};

const ENEMY_ONE_HEALTH: u8 = 5;

pub struct EnemyPlugin;

struct DicethulhuSheet(Handle<TextureAtlas>);
struct EnemyOneSheet(Handle<TextureAtlas>);

#[derive(Component)]
pub struct FacePlayer;
#[derive(Component, Debug)]
pub struct DiesToLaser;
#[derive(Component, Debug)]
pub struct Damaged;
#[derive(Component, Debug)]
pub struct Health {
    health: u8,
}
#[derive(Component, Debug)]
pub struct Invuln {
    timer: Timer,
}

#[derive(Component)]
pub struct Dicethulhu;
#[derive(Component)]
pub struct EnemyOne;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_dicethulhu)
            .add_startup_system(spawn_enemy_one)
            .add_system(face_player)
            .add_system(check_for_laser)
            .add_system(damaged)
            .add_system(animate_dicethulhu)
            .add_system(animate_enemy_one)
            .add_system(invuln)
            .add_system(enemy_one_movement);
    }
}

fn invuln(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TextureAtlasSprite, &mut Invuln)>,
    time: Res<Time>,
) {
    for (id, mut sprite, mut invuln) in query.iter_mut() {
        sprite.color.set_a(0.1);
        invuln.timer.tick(time.delta());
        if invuln.timer.just_finished() {
            sprite.color.set_a(1.0);
            commands.entity(id).remove::<Invuln>();
        }
    }
}

fn damaged(mut commands: Commands, mut query: Query<(Entity, &mut Health), Added<Damaged>>) {
    for (id, mut health) in query.iter_mut() {
        commands.entity(id).remove::<Damaged>().insert(Invuln {
            timer: Timer::new(Duration::from_millis(200), false),
        });
        health.health -= 1;
        if health.health == 0 {
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

// TODO
fn enemy_one_movement(mut _query: Query<&mut Transform, With<EnemyOne>>) {}

fn animate_enemy_one(
    mut query: Query<(&mut TextureAtlasSprite, &mut Transform), With<EnemyOne>>,
    time: Res<Time>,
) {
    for (mut enemy_sprite, mut enemy_transform) in query.iter_mut() {
        let frame = (time.time_since_startup().as_millis() / 100) % 7;
        enemy_sprite.index = frame.try_into().expect("Should always fit into u128");

        const TOTAL_DISPLACEMENT: f32 = 1.0;

        let frame = (frame + 1) % 7;
        let y_vel;

        if frame >= 2 {
            y_vel = -TOTAL_DISPLACEMENT / 5.0
        } else {
            y_vel = TOTAL_DISPLACEMENT / 2.0;
        }

        enemy_transform.translation.y += y_vel;
    }
}

fn spawn_enemy_one(mut commands: Commands, sprite_sheet: Res<EnemyOneSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(50.0, -60.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::cuboid(10.5, 8.0))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
        .insert(FacePlayer)
        .insert(DiesToLaser)
        .insert(EnemyOne)
        .insert(Health {
            health: ENEMY_ONE_HEALTH,
        })
        .insert(Name::from("EnemyOne"));
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
}
