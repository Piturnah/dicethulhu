use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod clouds;
mod debug;
mod enemy;
mod physics;
mod player;

use clouds::CloudsPlugin;
use debug::DebugPlugin;
use enemy::EnemyPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;

const RESOLUTION: f32 = 16.0 / 9.0;
const PIXEL_WIDTH: f32 = 320.0;

struct PlayerSheet(Handle<TextureAtlas>);
struct ArenaSprite(Handle<Image>);
struct SkyboxSprite(Handle<Image>);
struct BulletSprite(Handle<Image>);
struct GunSheet(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("LittleGuy.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::new(16.0, 21.0), 6, 2, Vec2::splat(2.0));
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(PlayerSheet(atlas_handle));

    let image = assets.load("Gun.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::new(23.0, 9.0), 2, 1, Vec2::splat(2.0));
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(GunSheet(atlas_handle));

    let image_handle = assets.load("Arena.png");
    commands.insert_resource(ArenaSprite(image_handle));
    let image_handle = assets.load("Skybox.png");
    commands.insert_resource(SkyboxSprite(image_handle));
    let image_handle = assets.load("Laser.png");
    commands.insert_resource(BulletSprite(image_handle));
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.2;

    commands.spawn_bundle(camera);
}

fn init_scene(
    mut commands: Commands,
    arena_texture: Res<ArenaSprite>,
    skybox_texture: Res<SkyboxSprite>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: arena_texture.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 50.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::from("Arena"));

    commands
        .spawn_bundle(SpriteBundle {
            texture: skybox_texture.0.clone(),
            ..Default::default()
        })
        .insert(Name::from("Skybox"));

    commands
        .spawn()
        .insert(Collider::cuboid(1.0, PIXEL_WIDTH / RESOLUTION))
        .insert(GlobalTransform::default())
        .insert(Transform {
            translation: Vec3::new(-(PIXEL_WIDTH / 2.0 + 1.0), 0.0, 0.0),
            ..Default::default()
        })
        .insert(Name::from("Left wall"));

    commands
        .spawn()
        .insert(Collider::cuboid(1.0, PIXEL_WIDTH / RESOLUTION))
        .insert(GlobalTransform::default())
        .insert(Transform {
            translation: Vec3::new(PIXEL_WIDTH / 2.0 + 1.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Name::from("Right wall"));

    commands
        .spawn()
        .insert(Collider::cuboid(PIXEL_WIDTH, 1.0))
        .insert(GlobalTransform::default())
        .insert(Transform {
            translation: Vec3::new(0.0, PIXEL_WIDTH / (RESOLUTION * 2.0) + 1.0, 0.0),
            ..Default::default()
        })
        .insert(Name::from("Ceiling"));
}

fn main() {
    let height = 900.0;

    App::new()
        .insert_resource(WindowDescriptor {
            width: height * RESOLUTION,
            height,
            title: "Dicethulhu!".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -500.0),
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(450.0))
        .add_plugin(DebugPlugin)
        .add_plugin(CloudsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
        .add_startup_system(spawn_camera)
        .add_startup_system(init_scene)
        .run();
}
