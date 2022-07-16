use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

use crate::{physics::GroundDetection, player::Player};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(RapierDebugRenderPlugin::default())
                .add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>()
                .register_inspectable::<GroundDetection>()
                .add_system(bevy::input::system::exit_on_esc_system);
        }
    }
}
