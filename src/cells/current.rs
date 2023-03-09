use bevy::{
    ecs::component::Component,
    math::{UVec2, UVec3},
};
use std::cmp::Ordering;

use crate::cells::cell::{Cell, Direction};

#[derive(Clone, Debug, Component)]
pub struct CurrentCells {
    pub main_cell: Cell,
    pub dimensions: UVec3,
    pub facing: Direction,
    pub underneath: Vec<Cell>,
    pub behind: Vec<Cell>,
}

impl CurrentCells {
    pub fn new(main_cell: Cell, dims: UVec3, facing: Direction, map_size: UVec2) -> Self {
        let underneath = Self::underneath(main_cell, dims, facing, map_size);
        let behind = Self::behind(&underneath, dims.z, map_size);
        Self {
            main_cell,
            dimensions: dims,
            facing,
            underneath,
            behind,
        }
    }

    // NOTE:
    // - main_cell is always the bottom-most cell
    // if facing BottomRight:
    // - dimensions.x expands towards TopRight
    // - dimensions.y expands towards TopLeft
    // if facing BottomLeft:
    // - dimensions.x expands towards TopLeft
    // - dimensions.y expands towards TopRight
    // Items can't face other directions
    fn underneath(main_cell: Cell, dims: UVec3, facing: Direction, map_size: UVec2) -> Vec<Cell> {
        if dims.x * dims.y == 1 {
            return vec![main_cell];
        }

        let (col_dir, row_dir) = match facing {
            Direction::BottomRight => (Direction::TopRight, Direction::TopLeft),
            Direction::BottomLeft => (Direction::TopLeft, Direction::TopRight),
            _ => panic!("Items can only face BottomRight or BottomLeft,\n{facing:?} is not valid"),
        };
        let mut underneath_cells = Vec::new();
        let mut current_cell = Some(main_cell);
        let mut current_row_cell = Some(main_cell);

        for _row in 0..dims.y {
            for col in 0..dims.x {
                let is_at_start_of_col = col == 0;
                if is_at_start_of_col {
                    current_cell = current_row_cell;
                }

                underneath_cells.push(current_cell);

                let has_found_all_cells = underneath_cells.len() == (dims.x * dims.y) as usize;
                if has_found_all_cells {
                    return Self::flatten(underneath_cells);
                }

                let is_at_end_of_col = col == dims.x - 1;
                if is_at_end_of_col {
                    current_row_cell =
                        current_row_cell.and_then(|cell| cell.next_cell(row_dir, map_size));
                } else {
                    current_cell = current_cell.and_then(|cell| cell.next_cell(col_dir, map_size));
                }
            }
        }
        Self::flatten(underneath_cells)
    }

    fn flatten(underneath_cells: Vec<Option<Cell>>) -> Vec<Cell> {
        underneath_cells
            .into_iter()
            .flatten()
            .collect::<Vec<Cell>>()
    }

    fn behind(underneath: &[Cell], height: u32, map_size: UVec2) -> Vec<Cell> {
        let mut behind_cells = Vec::new();
        let mut currently_checking = underneath.iter().map(Clone::clone).collect::<Vec<Cell>>();
        for _step in 0..height {
            let mut next_cells_to_check: Vec<Cell> = Vec::new();
            for check in &currently_checking {
                if let Some(top_left_cell) = check.next_cell(Direction::TopLeft, map_size) {
                    let is_underneath = underneath.contains(&top_left_cell);
                    if !behind_cells.contains(&top_left_cell) && !is_underneath {
                        behind_cells.push(top_left_cell);
                    }
                }
                if let Some(top_right_cell) = check.next_cell(Direction::TopRight, map_size) {
                    let is_underneath = underneath.contains(&top_right_cell);
                    if !behind_cells.contains(&top_right_cell) && !is_underneath {
                        behind_cells.push(top_right_cell);
                    }
                }
                if let Some(top_cell) = check.next_cell(Direction::Top, map_size) {
                    let is_underneath = underneath.contains(&top_cell);
                    if !behind_cells.contains(&top_cell) && !is_underneath {
                        behind_cells.push(top_cell);
                    }
                    if !next_cells_to_check.contains(&top_cell) && !is_underneath {
                        next_cells_to_check.push(top_cell);
                    }
                }
            }
            currently_checking = next_cells_to_check;
        }
        behind_cells
    }
}

impl PartialEq for CurrentCells {
    fn eq(&self, other: &Self) -> bool {
        self.main_cell == other.main_cell
            && self.dimensions == other.dimensions
            && self.facing == other.facing
    }
}

impl Eq for CurrentCells {}

impl PartialOrd for CurrentCells {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let is_other_behind_self = self
            .behind
            .iter()
            .any(|self_behind| other.underneath.contains(self_behind));

        let is_self_behind_other = other
            .behind
            .iter()
            .any(|other_behind| self.underneath.contains(other_behind));

        match (is_other_behind_self, is_self_behind_other) {
            (true, true) => panic!("Items cannot be both in front and behind each other"),
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (false, false) => None,
        }
    }
}
