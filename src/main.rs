use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

mod clouds;
mod debug;

use clouds::CloudsPlugin;
use debug::DebugPlugin;

const RESOLUTION: f32 = 16.0 / 9.0;

struct PlayerSheet(Handle<TextureAtlas>);
struct ArenaSprite(Handle<Image>);
struct SkyboxSprite(Handle<Image>);
struct CloudsSprite(Handle<Image>);
struct DicethulhuSprite(Handle<Image>);

#[derive(Inspectable)]
enum PlayerAnimState {
    Idle,
    Run,
    Jump,
}

#[derive(Component, Inspectable)]
struct Player {
    speed: f32,
    jump_force: f32,
    anim_state: PlayerAnimState,
}

#[derive(Component, Inspectable, Default)]
struct GroundDetection {
    grounded: bool,
}

#[derive(Component)]
struct GroundSensor {
    ground_detection_entity: Entity,
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

fn detect_ground(
    sensors: Query<&GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    mut entities: Query<&mut GroundDetection>,
) {
    for collision in collisions.iter() {
        match collision {
            CollisionEvent::Started(a, b, _) => {
                // TODO: Inspect order for entities in `CollisionEvent`, as it always seems to be
                // the sensor in `b`
                match sensors.get(*b) {
                    Ok(sensor) => match entities.get_mut(sensor.ground_detection_entity) {
                        Ok(mut entity) => {
                            entity.grounded = true;
                        }
                        Err(_) => {}
                    },
                    Err(_) => match sensors.get(*a) {
                        Ok(sensor) => match entities.get_mut(sensor.ground_detection_entity) {
                            Ok(mut entity) => {
                                entity.grounded = true;
                            }
                            Err(_) => {}
                        },
                        Err(_) => {}
                    },
                }
            }
            CollisionEvent::Stopped(a, b, _) => match sensors.get(*b) {
                Ok(sensor) => match entities.get_mut(sensor.ground_detection_entity) {
                    Ok(mut entity) => {
                        entity.grounded = false;
                    }
                    Err(_) => {}
                },
                Err(_) => match sensors.get(*a) {
                    Ok(sensor) => match entities.get_mut(sensor.ground_detection_entity) {
                        Ok(mut entity) => {
                            entity.grounded = false;
                        }
                        Err(_) => {}
                    },
                    Err(_) => {}
                },
            },
        }
    }
}

fn spawn_ground_sensor(
    mut commands: Commands,
    query: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
    const SENSOR_HEIGHT: f32 = 1.0;

    for (id, collider) in query.iter() {
        let collider_extents = collider
            .as_cuboid()
            .expect("All GroundDetection entities should use a cuboid collider")
            .half_extents();

        let sensor = commands
            .spawn()
            .insert(GroundSensor {
                ground_detection_entity: id,
            })
            .insert(Collider::cuboid(collider_extents[0], SENSOR_HEIGHT))
            .insert(Sensor)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Transform {
                translation: Vec3::new(0.0, -(collider_extents[1] + SENSOR_HEIGHT), 0.0),
                ..Default::default()
            })
            .insert(Name::from("Sensor"))
            .id();

        commands.entity(id).add_child(sensor);
    }
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

    let image_handle = assets.load("Arena.png");
    commands.insert_resource(ArenaSprite(image_handle));
    let image_handle = assets.load("Skybox.png");
    commands.insert_resource(SkyboxSprite(image_handle));
    let image_handle = assets.load("SkyboxClouds.png");
    commands.insert_resource(CloudsSprite(image_handle));
    let image_handle = assets.load("DicethulhuTest.png");
    commands.insert_resource(DicethulhuSprite(image_handle));
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -123.0, 0.0)))
        .insert(Name::from("Ground Collider"));
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
    dicethulhu_texture: Res<DicethulhuSprite>,
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
        .spawn_bundle(SpriteBundle {
            texture: dicethulhu_texture.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 40.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::from("Dicethulhu"));
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
        .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(init_scene)
        .add_startup_system(setup_physics)
        .add_system(player_movement)
        .add_system(animate_player)
        .add_system(spawn_ground_sensor)
        .add_system(detect_ground)
        .run();
}
