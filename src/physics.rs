use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

pub struct PhysicsPlugin;

#[derive(Component, Inspectable, Default)]
pub struct GroundDetection {
    pub grounded: bool,
}

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ground_collider)
            .add_system(detect_ground)
            .add_system(spawn_ground_sensor);
    }
}

fn spawn_ground_collider(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -123.0, 0.0)))
        .insert(Name::from("Ground Collider"));
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
