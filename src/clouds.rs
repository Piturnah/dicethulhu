use bevy::prelude::*;

pub struct CloudsPlugin;

const SPEED_NEAR: f32 = 5.0;
const SPEED_MIDDLE: f32 = 3.0;
const SPEED_FAR: f32 = 1.0;
const WIDTH: f32 = 320.0;

#[derive(Component)]
struct Clouds {
    speed: f32,
}

struct CloudsSheet(Handle<TextureAtlas>);

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(init_clouds)
            .add_system(scroll_clouds);
    }
}

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("SeparateClouds.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(320.0, 180.0),
        3,
        1,
        Vec2::splat(2.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(CloudsSheet(atlas_handle));
}

fn scroll_clouds(mut q: Query<(&mut Transform, &Clouds)>, time: Res<Time>) {
    for (mut transform, clouds) in q.iter_mut() {
        transform.translation.x -= clouds.speed * time.delta_seconds();
        if transform.translation.x < -WIDTH {
            transform.translation.x = -transform.translation.x;
        }
    }
}

fn init_clouds(mut commands: Commands, texture: Res<CloudsSheet>) {
    let mut cloud_tiles = Vec::with_capacity(3);

    for i in 0..3 {
        for j in 0..3 {
            cloud_tiles.push(
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(i),
                        texture_atlas: texture.0.clone(),
                        transform: Transform {
                            translation: Vec3::new(
                                -WIDTH + WIDTH * j as f32,
                                0.0,
                                30.0 + 1.0 * (2.0 - i as f32),
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Clouds {
                        speed: match i {
                            0 => SPEED_NEAR,
                            1 => SPEED_MIDDLE,
                            2 => SPEED_FAR,
                            _ => unreachable!(),
                        },
                    })
                    .insert(Name::from(format!("Clouds_{i}_{j}")))
                    .id(),
            );
        }
    }

    commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::default())
        .insert(Name::from("Clouds"))
        .push_children(&cloud_tiles);
}
