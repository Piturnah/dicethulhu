use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct EnemyPlugin;

struct DicethulhuSheet(Handle<TextureAtlas>);
struct EnemyOneSheet(Handle<TextureAtlas>);

#[derive(Component)]
pub struct Dicethulhu;
#[derive(Component)]
pub struct EnemyOne;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_dicethulhu)
            .add_startup_system(spawn_enemy_one)
            .add_system(animate_dicethulhu)
            .add_system(animate_enemy_one);
    }
}

fn animate_dicethulhu(
    mut query: Query<&mut TextureAtlasSprite, With<Dicethulhu>>,
    time: Res<Time>,
) {
    let mut dicethulhu_sprite = query.single_mut();
    dicethulhu_sprite.index = ((time.time_since_startup().as_millis() / 200) % 16)
        .try_into()
        .expect("Should always fit into u128");
}

fn spawn_dicethulhu(mut commands: Commands, sprite_sheet: Res<DicethulhuSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 40.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Dicethulhu)
        .insert(Name::from("Dicethulhu"));
}

fn animate_enemy_one(mut query: Query<&mut TextureAtlasSprite, With<EnemyOne>>, time: Res<Time>) {
    let mut enemy_sprite = query.single_mut();
    enemy_sprite.index = ((time.time_since_startup().as_millis() / 100) % 7)
        .try_into()
        .expect("Should always fit into u128");
}

fn spawn_enemy_one(mut commands: Commands, sprite_sheet: Res<EnemyOneSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(50.0, 0.0, 100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider::cuboid(10.5, 8.0))
        .insert(EnemyOne)
        .insert(Name::from("EnemyOne"));
}

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Dicethulhu.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(320.0, 180.0),
        16,
        1,
        Vec2::splat(2.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(DicethulhuSheet(atlas_handle));

    let image = assets.load("Enemy1.png");
    let atlas =
        TextureAtlas::from_grid_with_padding(image, Vec2::new(21.0, 16.0), 7, 2, Vec2::splat(2.0));
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(EnemyOneSheet(atlas_handle));
}
