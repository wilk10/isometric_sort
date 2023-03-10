use bevy::{
    ecs::{component::Component, entity::Entity, reflect::ReflectComponent},
    math::UVec3,
    reflect::Reflect,
    utils::HashMap,
};

use crate::cells::{
    cell::{Cell, Direction},
    current::CurrentCells,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SavedCells {
    pub main_cell: Cell,
    pub dimensions: UVec3,
    pub facing: Direction,
}

impl Default for SavedCells {
    fn default() -> Self {
        Self {
            main_cell: Cell::new(0, 0),
            dimensions: UVec3::ONE,
            facing: Direction::BottomRight,
        }
    }
}

impl From<&CurrentCells> for SavedCells {
    fn from(cells: &CurrentCells) -> Self {
        Self {
            main_cell: cells.main_cell,
            dimensions: cells.dimensions,
            facing: cells.facing,
        }
    }
}

#[derive(Component)]
pub struct Mistake;

#[derive(Debug, Component)]
pub struct EntitiesNearby {
    pub corresponding: Entity,
    pub in_front: Vec<Entity>,
    pub behind: Vec<Entity>,
}

#[derive(Debug, Component)]
pub struct CompareTransforms {
    pub map: HashMap<SortMethod, f32>,
}

impl Default for CompareTransforms {
    fn default() -> Self {
        Self {
            map: SortMethod::all()
                .iter()
                .fold(HashMap::new(), |mut map, method| {
                    map.insert(*method, 0.);
                    map
                }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortMethod {
    Topological,
    PartialCmp,
}

impl SortMethod {
    pub fn all() -> [Self; 2] {
        [Self::Topological, Self::PartialCmp]
    }
}
