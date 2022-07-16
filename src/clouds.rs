use bevy::prelude::*;

use crate::CloudsSprite;

pub struct CloudsPlugin;

const SPEED: f32 = 5.0;
const WIDTH: f32 = 320.0;

#[derive(Component)]
struct Clouds;

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_clouds)
            .add_system(scroll_clouds);
    }
}

fn scroll_clouds(mut q: Query<&mut Transform, With<Clouds>>, time: Res<Time>) {
    for mut clouds in q.iter_mut() {
        clouds.translation.x -= SPEED * time.delta_seconds();
	if clouds.translation.x < -WIDTH {
	    clouds.translation.x = -clouds.translation.x;
	}
    }
}

fn init_clouds(mut commands: Commands, texture: Res<CloudsSprite>) {
    let mut cloud_tiles = Vec::with_capacity(3);

    for i in 0..3 {
        cloud_tiles.push(
            commands
                .spawn_bundle(SpriteBundle {
                    texture: texture.0.clone(),
                    transform: Transform {
                        translation: Vec3::new(-WIDTH + WIDTH * i as f32, 0.0, 30.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Clouds)
                .insert(Name::from(format!("Clouds_{i}")))
                .id(),
        );
    }

    commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::default())
        .insert(Name::from("Clouds"))
        .push_children(&cloud_tiles);
}
