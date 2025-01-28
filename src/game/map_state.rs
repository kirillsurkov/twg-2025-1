use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

pub struct MapStatePlugin;

impl Plugin for MapStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapState::default())
            .add_systems(PreUpdate, check_connectivity);
    }
}

#[derive(PartialEq)]
enum MapNode {
    PrimaryBlock,
    Room(bool),
}

#[derive(Resource, Default)]
pub struct MapState {
    map: HashMap<(i32, i32), MapNode>,
    temp_disconnects: HashSet<(i32, i32)>,
}

impl MapState {
    fn add_block(&mut self, x: i32, y: i32, node: MapNode) {
        self.map.insert((x, y), node);
    }

    pub fn add_primary_block(&mut self, x: i32, y: i32) {
        self.add_block(x, y, MapNode::PrimaryBlock);
    }

    pub fn add_room(&mut self, x: i32, y: i32) {
        self.add_block(x, y, MapNode::Room(true));
    }

    pub fn remove(&mut self, x: i32, y: i32) {
        self.map.remove(&(x, y));
    }

    pub fn add_temp_disconnect(&mut self, x: i32, y: i32) {
        self.temp_disconnects.insert((x, y));
    }

    pub fn is_available(&self, x: i32, y: i32) -> bool {
        !self.map.contains_key(&(x, y))
            && (self.map.contains_key(&(x + 1, y))
                || self.map.contains_key(&(x - 1, y))
                || self.map.contains_key(&(x, y + 1))
                || self.map.contains_key(&(x, y - 1)))
    }

    pub fn is_room(&self, x: i32, y: i32) -> bool {
        match self.map.get(&(x, y)) {
            Some(MapNode::Room(_)) => true,
            _ => false,
        }
    }

    pub fn is_room_connected(&self, x: i32, y: i32) -> bool {
        match self.map.get(&(x, y)) {
            Some(MapNode::Room(connected)) => *connected,
            _ => false,
        }
    }
}

fn check_connectivity(mut map_state: ResMut<MapState>) {
    let primary_blocks = map_state
        .map
        .iter_mut()
        .filter_map(|(c, n)| match n {
            MapNode::PrimaryBlock => Some(c.clone()),
            MapNode::Room(connected) => {
                *connected = false;
                None
            }
        })
        .collect::<HashSet<_>>();

    let mut stack = primary_blocks.iter().cloned().collect::<Vec<_>>();

    let mut visited = map_state
        .temp_disconnects
        .iter()
        .cloned()
        .filter(|c| !primary_blocks.contains(c))
        .collect::<HashSet<_>>();

    while let Some((x, y)) = stack.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }
        visited.insert((x, y));

        match map_state.map.get_mut(&(x, y)).unwrap() {
            MapNode::Room(connected) => *connected = true,
            _ => {}
        }

        if map_state.is_room(x + 1, y) {
            stack.push((x + 1, y));
        }
        if map_state.is_room(x - 1, y) {
            stack.push((x - 1, y));
        }
        if map_state.is_room(x, y + 1) {
            stack.push((x, y + 1));
        }
        if map_state.is_room(x, y - 1) {
            stack.push((x, y - 1));
        }
    }

    map_state.temp_disconnects.clear();
}
