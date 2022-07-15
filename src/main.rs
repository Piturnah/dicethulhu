use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

mod debug;

use debug::DebugPlugin;

const RESOLUTION: f32 = 16.0 / 9.0;

struct AsciiSheet(pub Handle<TextureAtlas>);

#[derive(Component, Inspectable)]
struct Player {speed: f32}

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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(450.0))
        .add_plugin(DebugPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_ascii)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(setup_physics)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(player_movement)
        .run();
}

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();

    let mut delta = 0.0;
    if keyboard.pressed(KeyCode::D) {
	delta += player.speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::A) {
	delta -= player.speed * time.delta_seconds();
    }

    transform.translation += Vec3::new(delta, 0.0, 0.0);
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -200.0, 0.0)));
}

fn spawn_player(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let mut sprite = TextureAtlasSprite::new(1);
    sprite.custom_size = Some(Vec2::splat(50.0));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: ascii.0.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(50.0, 50.0))
        .insert(Player {speed: 200.0} )
        .insert(Name::new("Player"));
}

fn load_ascii(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Ascii.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::splat(9.0), 16, 16, Vec2::splat(2.0));

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(AsciiSheet(atlas_handle))
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
