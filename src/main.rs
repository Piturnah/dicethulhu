use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

mod debug;

use debug::DebugPlugin;

const RESOLUTION: f32 = 16.0 / 9.0;

struct PlayerSheet(pub Handle<TextureAtlas>);

#[derive(Inspectable)]
enum PlayerAnimState {
    Idle,
    Run,
}

#[derive(Component, Inspectable)]
struct Player {
    speed: f32,
    anim_state: PlayerAnimState,
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(450.0))
        .add_plugin(DebugPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(setup_physics)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(player_movement)
        .add_system(animate_player)
        .run();
}

fn player_movement(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform) = player_query.single_mut();

    let mut delta = 0.0;
    if keyboard.pressed(KeyCode::D) {
        delta += player.speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::A) {
        delta -= player.speed * time.delta_seconds();
    }

    if delta != 0.0 {
        player.anim_state = PlayerAnimState::Run;
    } else {
        player.anim_state = PlayerAnimState::Idle;
    }

    transform.translation += Vec3::new(delta, 0.0, 0.0);
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));
}

fn spawn_player(mut commands: Commands, sprite_sheet: Res<PlayerSheet>) {
    let sprite = TextureAtlasSprite::new(0);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: sprite_sheet.0.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(8.0, 10.5))
        .insert(Player {
            speed: 100.0,
            anim_state: PlayerAnimState::Idle,
        })
        .insert(Name::new("Player"));
}

fn animate_player(mut commands: Commands, player_query: Query<(Entity, &Player)>, time: Res<Time>) {
    let (id, player) = player_query.single();

    commands.entity(id).remove::<TextureAtlasSprite>();
    commands
        .entity(id)
        .insert(TextureAtlasSprite::new(match player.anim_state {
            PlayerAnimState::Idle => 0,
            PlayerAnimState::Run => ((time.time_since_startup().as_millis() / 100) % 5 + 1)
                .try_into()
                .expect("Spritesheet index should always fit into usize!"),
        }));
}

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("LittleGuy.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::new(16.0, 21.0), 6, 1, Vec2::splat(2.0));

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(PlayerSheet(atlas_handle))
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.2;

    commands.spawn_bundle(camera);
}
