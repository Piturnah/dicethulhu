use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

use crate::Player;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(RapierDebugRenderPlugin::default())
                .add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>();
        }
    }
}
