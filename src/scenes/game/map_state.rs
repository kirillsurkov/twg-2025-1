use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::scenes::AppState;

pub struct MapStatePlugin;

impl Plugin for MapStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapState::default()).add_systems(
            PreUpdate,
            check_connectivity.run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MapNode {
    PrimaryBlock,
    EmptyRoom,
    Furnace,
    Generator,
    Crusher,
    Cargo,
    Hook,
    Enrichment,
}

impl MapNode {
    pub fn thumbnail(&self) -> &str {
        match self {
            MapNode::PrimaryBlock => "primary.png",
            MapNode::EmptyRoom => "room.png",
            MapNode::Furnace => "furnace.png",
            MapNode::Generator => "generator.png",
            MapNode::Crusher => "crusher.png",
            MapNode::Cargo => "cargo.png",
            MapNode::Hook => "hook.png",
            MapNode::Enrichment => "enrichment.png",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MapNode::PrimaryBlock => "Main block",
            MapNode::EmptyRoom => "Empty room",
            MapNode::Furnace => "Furnace",
            MapNode::Generator => "Generator",
            MapNode::Crusher => "Crusher",
            MapNode::Cargo => "Cargo",
            MapNode::Hook => "Hook",
            MapNode::Enrichment => "Enrichment station",
        }
    }

    pub fn desc(&self) -> &str {
        match self {
            MapNode::PrimaryBlock => "Your rescue capsule",
            MapNode::EmptyRoom => "Just an empty room",
            MapNode::Furnace => "Melts ores and ice",
            MapNode::Generator => "Generates power",
            MapNode::Crusher => "Crushes stones into the silicone dust",
            MapNode::Cargo => "Increases your storage capabilities",
            MapNode::Hook => "Automatic hook",
            MapNode::Enrichment => "Produces batteries",
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum MapLayer {
    Main,
    Build,
}

#[derive(Resource, Default)]
pub struct MapState {
    map_by_layer: HashMap<MapLayer, HashMap<IVec2, MapNode>>,
    bounds_min: IVec2,
    bounds_max: IVec2,
}

impl MapState {
    fn recalculate_bounds(&mut self) {
        self.bounds_min = IVec2::MAX;
        self.bounds_max = IVec2::MIN;
        for (_, map) in &self.map_by_layer {
            for (vec, _) in map {
                self.bounds_min = self.bounds_min.min(*vec);
                self.bounds_max = self.bounds_max.max(*vec);
            }
        }
    }

    fn add(&mut self, x: i32, y: i32, node: MapNode, layer: MapLayer) {
        self.map_by_layer
            .entry(layer)
            .or_default()
            .insert(IVec2::new(x, y), node);
        self.recalculate_bounds();
    }

    fn remove(&mut self, x: i32, y: i32, layer: MapLayer) {
        self.map_by_layer
            .entry(layer)
            .or_default()
            .remove(&IVec2::new(x, y));
        self.recalculate_bounds();
    }

    pub fn add_primary_block(&mut self, x: i32, y: i32) {
        self.add(x, y, MapNode::PrimaryBlock, MapLayer::Main);
    }

    pub fn add_room(&mut self, x: i32, y: i32, node: MapNode) {
        self.add(x, y, node, MapLayer::Main);
    }

    pub fn remove_room(&mut self, x: i32, y: i32, layer: MapLayer) {
        if self.is_room(x, y, layer.clone()) {
            self.remove(x, y, layer);
        }
    }

    pub fn sync_build(&mut self) {
        let main_layer = self
            .map_by_layer
            .get(&MapLayer::Main)
            .cloned()
            .unwrap_or_default();
        self.map_by_layer.insert(MapLayer::Build, main_layer);
    }

    pub fn is_available(&self, x: i32, y: i32, node: MapNode) -> bool {
        let Some(map) = self.map_by_layer.get(&MapLayer::Main) else {
            return false;
        };
        match node {
            MapNode::EmptyRoom => {
                !map.contains_key(&IVec2::new(x, y))
                    && (map.contains_key(&IVec2::new(x + 1, y))
                        || map.contains_key(&IVec2::new(x - 1, y))
                        || map.contains_key(&IVec2::new(x, y + 1))
                        || map.contains_key(&IVec2::new(x, y - 1)))
            }
            _ => match map.get(&IVec2::new(x, y)) {
                Some(MapNode::EmptyRoom) => true,
                _ => false,
            },
        }
    }

    pub fn is_room(&self, x: i32, y: i32, layer: MapLayer) -> bool {
        self.map_by_layer
            .get(&layer)
            .is_some_and(|m| match m.get(&IVec2::new(x, y)) {
                Some(MapNode::PrimaryBlock) => false,
                Some(_) => true,
                _ => false,
            })
    }

    pub fn is_node(&self, x: i32, y: i32, layer: MapLayer) -> bool {
        self.map_by_layer
            .get(&layer)
            .is_some_and(|m| m.contains_key(&IVec2::new(x, y)))
    }

    pub fn node(&self, x: i32, y: i32, layer: MapLayer) -> Option<MapNode> {
        self.map_by_layer
            .get(&layer)
            .and_then(|m| m.get(&IVec2::new(x, y)))
            .cloned()
    }

    pub fn get_bounds(&self) -> (IVec2, IVec2) {
        (self.bounds_min, self.bounds_max)
    }

    pub fn primary_blocks(&self) -> Vec<IVec2> {
        if let Some(map) = self.map_by_layer.get(&MapLayer::Main) {
            map.iter()
                .filter_map(|(c, n)| match n {
                    MapNode::PrimaryBlock => Some(*c),
                    _ => None,
                })
                .collect()
        } else {
            vec![]
        }
    }
}

fn check_connectivity(mut map_state: ResMut<MapState>) {
    for (_, map) in &mut map_state.map_by_layer {
        let mut new_map = map
            .iter()
            .filter_map(|(c, n)| match n {
                MapNode::PrimaryBlock => Some((*c, MapNode::PrimaryBlock)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let mut stack = new_map.keys().cloned().collect::<Vec<_>>();

        let mut visited = HashSet::new();

        while let Some(IVec2 { x, y }) = stack.pop() {
            if !visited.insert(IVec2::new(x, y)) {
                continue;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let x = x + dx;
                let y = y + dy;
                if let Some(node) = map.get(&IVec2::new(x, y)) {
                    stack.push(IVec2::new(x, y));
                    new_map.insert(IVec2::new(x, y), node.clone());
                }
            }
        }

        *map = new_map;
    }
}
