use bevy::prelude::*;
use std::time::Duration;

pub struct HealthPlugin;

#[derive(Component, Debug)]
pub struct Damaged;
#[derive(Component, Debug)]
pub struct Health {
    pub health: u8,
}
#[derive(Component, Debug)]
pub struct Invuln {
    timer: Timer,
}

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(invuln).add_system(damaged);
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

fn damaged(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health), (Added<Damaged>, Without<Invuln>)>,
) {
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
