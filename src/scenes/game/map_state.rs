use std::collections::BTreeMap;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use strum_macros::EnumIter;

use crate::scenes::AppState;

pub struct MapStatePlugin;

impl Plugin for MapStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapState::default()).add_systems(
            PreUpdate,
            (tick, check_connectivity).run_if(in_state(AppState::Game)),
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Cargo {
    Stone,
    Silicon,
    Ice,
    Copper,
    Uranium,
    Aurelium,
    Water,
    CopperPlates,
    UraniumRods,
    Batteries,
}

impl Cargo {
    pub fn name(&self) -> &str {
        match self {
            Cargo::Stone => "Stone",
            Cargo::Silicon => "Silicon",
            Cargo::Ice => "Ice",
            Cargo::Copper => "Copper",
            Cargo::Uranium => "Uranium",
            Cargo::Aurelium => "Aurelium",
            Cargo::Water => "Water",
            Cargo::CopperPlates => "Copper plates",
            Cargo::UraniumRods => "Uranium rods",
            Cargo::Batteries => "Batteries",
        }
    }
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

    pub fn recipe(&self) -> BTreeMap<Cargo, u32> {
        match self {
            MapNode::PrimaryBlock => vec![],
            MapNode::EmptyRoom => vec![(Cargo::Silicon, 10)],
            MapNode::Furnace => vec![(Cargo::Silicon, 10), (Cargo::Uranium, 1)],
            MapNode::Generator => vec![(Cargo::Silicon, 10), (Cargo::UraniumRods, 5)],
            MapNode::Crusher => vec![(Cargo::Silicon, 30), (Cargo::CopperPlates, 20)],
            MapNode::Cargo => vec![(Cargo::CopperPlates, 20)],
            MapNode::Hook => vec![(Cargo::Silicon, 50)],
            MapNode::Enrichment => vec![(Cargo::Silicon, 100), (Cargo::UraniumRods, 30)],
        }
        .into_iter()
        .collect()
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
    total_energy: f32,
    cargo: HashMap<Cargo, f32>,
    cargo_max: HashMap<Cargo, f32>,
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

    pub fn cargo_count(&self, cargo: Cargo) -> (f32, f32) {
        (
            self.cargo.get(&cargo).cloned().unwrap_or_default(),
            self.cargo_max.get(&cargo).cloned().unwrap_or_default(),
        )
    }
}

fn tick(mut map_state: ResMut<MapState>, time: Res<Time>) {
    let Some(map) = map_state.map_by_layer.get(&MapLayer::Main).cloned() else {
        return;
    };

    let mut total_energy = 0.0;
    let mut cargo_max = HashMap::<Cargo, f32>::new();
    for node in map.values() {
        match node {
            MapNode::PrimaryBlock => {
                total_energy += 100.0;
                *cargo_max.entry(Cargo::Silicon).or_default() += 10.0;
                *cargo_max.entry(Cargo::Ice).or_default() += 10.0;
                *cargo_max.entry(Cargo::Copper).or_default() += 10.0;
                *cargo_max.entry(Cargo::Uranium).or_default() += 1.0;
            }
            MapNode::EmptyRoom => {
                total_energy -= 5.0;
            }
            MapNode::Furnace => {
                total_energy -= 20.0;
            }
            MapNode::Generator => {
                total_energy += 100.0;
            }
            MapNode::Crusher => {
                total_energy -= 10.0;
            }
            MapNode::Cargo => {
                *cargo_max.entry(Cargo::Stone).or_default() += 10.0;
                *cargo_max.entry(Cargo::Silicon).or_default() += 10.0;
                *cargo_max.entry(Cargo::Ice).or_default() += 10.0;
                *cargo_max.entry(Cargo::Copper).or_default() += 10.0;
                *cargo_max.entry(Cargo::Uranium).or_default() += 10.0;
                *cargo_max.entry(Cargo::Aurelium).or_default() += 1.0;
                *cargo_max.entry(Cargo::Water).or_default() += 10.0;
                *cargo_max.entry(Cargo::CopperPlates).or_default() += 10.0;
                *cargo_max.entry(Cargo::UraniumRods).or_default() += 5.0;
                *cargo_max.entry(Cargo::Batteries).or_default() += 1.0;
            }
            MapNode::Hook => {
                total_energy -= 50.0;
            }
            MapNode::Enrichment => {
                total_energy -= 200.0;
            }
        }
    }

    for (cargo, count) in &mut map_state.cargo {
        *count = count.min(cargo_max.get(cargo).cloned().unwrap_or_default());
    }

    map_state.cargo_max = cargo_max;
    map_state.total_energy = total_energy;
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
