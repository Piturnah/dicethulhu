use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::time::Duration;

use crate::{player::Player, ui::UpdatedHealth};

pub struct HealthPlugin;

#[derive(Component, Debug)]
pub struct Damaged;
#[derive(Component, Debug, Inspectable)]
pub struct Health {
    pub health: u8,
}
#[derive(Component, Debug)]
pub struct Invuln {
    duration: Timer,
    flash_period: Timer,
}

impl Default for Invuln {
    fn default() -> Self {
        Self {
            duration: Timer::from_seconds(1.0, false),
            flash_period: Timer::new(Duration::from_millis(200), true),
        }
    }
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
        invuln.duration.tick(time.delta());
        invuln.flash_period.tick(time.delta());

        // This is broken but ran out of time in jam
        if invuln.flash_period.just_finished() {
            let alpha = sprite.color.a();
            sprite.color.set_a(match alpha {
                0.1 => 0.0,
                0.0 | 1.0 => 0.1,
                _ => 1.0,
            });
        }
        if invuln.duration.just_finished() {
            sprite.color.set_a(1.0);
            commands.entity(id).remove::<Invuln>();
        }
    }
}

fn damaged(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health), Added<Damaged>>,
    player_query: Query<Entity, With<Player>>,
) {
    for (id, mut health) in query.iter_mut() {
        commands.entity(id).remove::<Damaged>();

        // Very hacky, but running out of time
        if player_query.get(id).is_ok() {
            commands.entity(id).insert(Invuln::default());
        } else {
            commands.entity(id).insert(Invuln {
                duration: Timer::new(Duration::from_millis(200), false),
                flash_period: Timer::new(Duration::from_millis(40), false),
            });
        }

        health.health -= 1;
        if health.health == 0 {
            commands.entity(id).despawn_recursive();
        }

        // Very hacky, but running out of time
        if let Ok(player) = player_query.get(id) {
            commands.entity(player).insert(UpdatedHealth);
        }
    }
}
