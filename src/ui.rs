use bevy::prelude::*;

use crate::{health::Health, player::Player, GameState};

pub struct UiPlugin;

const HEART_WIDTH: f32 = 18.0;

struct HeartsSheet(Handle<TextureAtlas>);
struct AsciiSheet(Handle<TextureAtlas>);

#[derive(Component)]
pub struct UpdatedHealth;
#[derive(Component)]
struct UiHeart;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_system_set(
                SystemSet::on_update(GameState::Play).with_system(render_player_health),
            );
    }
}

fn render_player_health(
    mut commands: Commands,
    heart_sheet: Res<HeartsSheet>,
    query: Query<(Entity, &Health), (With<Player>, Added<UpdatedHealth>)>,
    hearts: Query<Entity, With<UiHeart>>,
) {
    const PADDING: f32 = 5.0;

    let pos = Vec3::new(146.0, 76.0, 200.0);
    if let Ok((id, health)) = query.get_single() {
        for heart in hearts.iter() {
            commands.entity(heart).despawn_recursive();
        }

        for i in 0..health.health {
            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(0),
                    texture_atlas: heart_sheet.0.clone(),
                    transform: Transform {
                        translation: pos - Vec3::new((HEART_WIDTH + PADDING) * i as f32, 0.0, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(UiHeart)
                .insert(Name::from("Heart"));

            commands.entity(id).remove::<UpdatedHealth>();
        }
    }
}

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Hearts.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(HEART_WIDTH, HEART_WIDTH),
        3,
        1,
        Vec2::splat(2.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(HeartsSheet(atlas_handle));
}
