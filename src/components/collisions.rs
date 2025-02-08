use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_rapier2d::prelude::*;

pub struct CollisionsPlugin;

impl Plugin for CollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Collisions::default())
            .add_systems(Update, update);
    }
}

#[derive(Resource, Default)]
pub struct Collisions {
    map: HashMap<Entity, HashSet<Entity>>,
    dummy: HashSet<Entity>,
}

impl Collisions {
    pub fn get(&self, entity: Entity) -> &HashSet<Entity> {
        self.map.get(&entity).unwrap_or(&self.dummy)
    }
}

fn update(mut collision_events: EventReader<CollisionEvent>, mut collisions: ResMut<Collisions>) {
    for event in collision_events.read() {
        match *event {
            CollisionEvent::Started(e1, e2, _) => {
                collisions.map.entry(e1).or_default().insert(e2);
                collisions.map.entry(e2).or_default().insert(e1);
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                collisions.map.entry(e1).or_default().remove(&e2);
                collisions.map.entry(e2).or_default().remove(&e1);
            }
        }
    }
}
