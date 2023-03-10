use bevy::{
    ecs::{component::Component, reflect::ReflectComponent},
    math::UVec3,
    reflect::Reflect,
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
