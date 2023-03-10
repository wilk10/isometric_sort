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

    pub fn prod_dims(&self) -> u32 {
        self.dimensions.x * self.dimensions.y * self.dimensions.z
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

#[cfg(test)]
mod test_cells_underneath {
    use super::*;

    /*
      |   |   |
    |0,0|1,0|2,0|
      |0,1|1,1|2,1|
    |0,2|1,2|2,2|
      |0,3|1,3|2,3|
    |0,4|1,4|2,4|
      |0,5|1,5|2,5|
        |   |   |
    */

    #[test]
    fn test_cells_underneath_1x1_cell_13() {
        let main_cell = Cell::new(1, 3);

        let expected = vec![main_cell];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 1, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_1x1_cell_01() {
        let main_cell = Cell::new(0, 1);

        let expected = vec![main_cell];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 1, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_1x1_cell_02() {
        let main_cell = Cell::new(0, 2);

        let expected = vec![main_cell];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 1, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_2x2() {
        let main_cell = Cell::new(1, 3);

        let expected = vec![main_cell, Cell::new(2, 2), Cell::new(1, 2), Cell::new(1, 1)];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(2, 2, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_1x2_facing_bottom_right() {
        let main_cell = Cell::new(1, 3);

        let expected = vec![main_cell, Cell::new(1, 2)];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 2, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_1x2_facing_bottom_left() {
        let main_cell = Cell::new(1, 3);

        let expected = vec![main_cell, Cell::new(2, 2)];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 2, 1),
            Direction::BottomLeft,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_2x1_facing_bottom_right() {
        let main_cell = Cell::new(2, 4);

        let expected = vec![main_cell, Cell::new(2, 3)];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(2, 1, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_2x1_facing_bottom_left() {
        let main_cell = Cell::new(2, 4);

        let expected = vec![main_cell, Cell::new(1, 3)];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(2, 1, 1),
            Direction::BottomLeft,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_2x3_facing_bottom_right() {
        let main_cell = Cell::new(1, 5);

        let expected = vec![
            main_cell,
            Cell::new(2, 4),
            Cell::new(1, 4),
            Cell::new(1, 3),
            Cell::new(0, 3),
            Cell::new(1, 2),
        ];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(2, 3, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_2x3_facing_bottom_left() {
        let main_cell = Cell::new(0, 3);

        let expected = vec![
            main_cell,
            Cell::new(0, 2),
            Cell::new(1, 2),
            Cell::new(0, 1),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(2, 3, 1),
            Direction::BottomLeft,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_3x2_facing_bottom_right() {
        let main_cell = Cell::new(1, 4);

        let expected = vec![
            main_cell,
            Cell::new(1, 3),
            Cell::new(2, 2),
            Cell::new(0, 3),
            Cell::new(1, 2),
            Cell::new(1, 1),
        ];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(3, 2, 1),
            Direction::BottomRight,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cells_underneath_3x2_facing_bottom_left() {
        let main_cell = Cell::new(1, 3);

        let expected = vec![
            main_cell,
            Cell::new(1, 2),
            Cell::new(0, 1),
            Cell::new(2, 2),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];

        let actual = CurrentCells::underneath(
            main_cell,
            UVec3::new(3, 2, 1),
            Direction::BottomLeft,
            UVec2::new(3, 6),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_cells_underneath_invalid_facing_direction() {
        let main_cell = Cell::new(1, 1);

        CurrentCells::underneath(
            main_cell,
            UVec3::new(1, 2, 1),
            Direction::Top,
            UVec2::new(3, 6),
        );
    }

    #[test]
    fn test_cells_underneath_too_close_to_map_border() {
        let main_cell = Cell::new(1, 2);
        let dims = UVec3::new(3, 2, 1);

        let actual =
            CurrentCells::underneath(main_cell, dims, Direction::BottomLeft, UVec2::new(3, 6));

        assert_ne!(actual.len(), (dims.x * dims.y) as usize);
    }
}

#[cfg(test)]
mod test_behind_cells {
    use super::*;

    /*
      |   |   |
    |0,0|1,0|2,0|
      |0,1|1,1|2,1|
    |0,2|1,2|2,2|
      |0,3|1,3|2,3|
    |0,4|1,4|2,4|
      |0,5|1,5|2,5|
    |0,6|1,6|2,6|
      |   |   |
    */

    #[test]
    fn test_behind_1x1x1_even_y() {
        let main_cell = Cell::new(1, 2);
        let expected = vec![Cell::new(0, 1), Cell::new(1, 1), Cell::new(1, 0)];
        let actual = CurrentCells::behind(&[main_cell], 1, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_1x1x1_odd_y() {
        let main_cell = Cell::new(1, 5);
        let expected = vec![Cell::new(1, 4), Cell::new(2, 4), Cell::new(1, 3)];
        let actual = CurrentCells::behind(&[main_cell], 1, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_1x1x2_even_y() {
        let main_cell = Cell::new(1, 4);
        let expected = vec![
            Cell::new(0, 3),
            Cell::new(1, 3),
            Cell::new(1, 2),
            Cell::new(0, 1),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];
        let actual = CurrentCells::behind(&[main_cell], 2, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_1x1x2_odd_y() {
        let main_cell = Cell::new(0, 5);
        let expected = vec![
            Cell::new(0, 4),
            Cell::new(1, 4),
            Cell::new(0, 3),
            Cell::new(0, 2),
            Cell::new(1, 2),
            Cell::new(0, 1),
        ];
        let actual = CurrentCells::behind(&[main_cell], 2, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_1x1x3() {
        let main_cell = Cell::new(1, 6);
        let expected = vec![
            Cell::new(0, 5),
            Cell::new(1, 5),
            Cell::new(1, 4),
            Cell::new(0, 3),
            Cell::new(1, 3),
            Cell::new(1, 2),
            Cell::new(0, 1),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];
        let actual = CurrentCells::behind(&[main_cell], 3, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_2x2x1() {
        let underneath = vec![
            Cell::new(1, 4),
            Cell::new(0, 3),
            Cell::new(1, 3),
            Cell::new(1, 2),
        ];
        let expected = vec![
            Cell::new(0, 2),
            Cell::new(0, 1),
            Cell::new(2, 2),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];
        let actual = CurrentCells::behind(&underneath, 1, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_2x2x2() {
        let underneath = vec![
            Cell::new(1, 6),
            Cell::new(0, 5),
            Cell::new(1, 5),
            Cell::new(1, 4),
        ];
        let expected = vec![
            Cell::new(0, 4),
            Cell::new(0, 3),
            Cell::new(2, 4),
            Cell::new(1, 3),
            Cell::new(1, 2),
            Cell::new(0, 2),
            Cell::new(0, 1),
            Cell::new(2, 2),
            Cell::new(1, 1),
            Cell::new(1, 0),
        ];
        let actual = CurrentCells::behind(&underneath, 2, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_2x1x2() {
        let underneath = vec![Cell::new(1, 5), Cell::new(2, 4)];
        let expected = vec![
            Cell::new(1, 4),
            Cell::new(1, 3),
            Cell::new(2, 3),
            Cell::new(2, 2),
            Cell::new(1, 2),
            Cell::new(1, 1),
            Cell::new(2, 1),
            Cell::new(2, 0),
        ];
        let actual = CurrentCells::behind(&underneath, 2, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_behind_1x2x2() {
        let underneath = vec![Cell::new(1, 6), Cell::new(0, 5)];
        let expected = vec![
            Cell::new(1, 5),
            Cell::new(1, 4),
            Cell::new(0, 4),
            Cell::new(0, 3),
            Cell::new(1, 3),
            Cell::new(1, 2),
            Cell::new(0, 2),
            Cell::new(0, 1),
        ];
        let actual = CurrentCells::behind(&underneath, 2, UVec2::new(3, 7));
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod test_sort_item {
    use bevy::ecs::world::World;

    use super::*;

    /*
      |   |   |
    |0,0|1,0|2,0|
      |0,1|1,1|2,1|
    |0,2|1,2|2,2|
      |0,3|1,3|2,3|
    |0,4|1,4|2,4|
      |0,5|1,5|2,5|
    |0,6|1,6|2,6|
      |   |   |
    */

    // TODO: remove the use of World and spawning the entity
    fn setup(world: &mut World, cell: Cell, dims: UVec3) -> CurrentCells {
        let _item_entity = world.spawn_empty().id();
        CurrentCells::new(cell, dims, Direction::BottomRight, UVec2::new(3, 7))
    }

    /*
      |   |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   | B |   |
    |   | A |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_1x1x1_vs_1x1x1_a_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(1, 4), UVec3::new(1, 1, 1));
        let b = setup(&mut world, Cell::new(1, 3), UVec3::new(1, 1, 1));
        assert!(a > b);
    }

    /*
      |   |   |
    |   |   |   |
      |   |   |   |
    |   |   | A |
      |   | B |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_1x1x1_vs_1x1x1_b_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(2, 2), UVec3::new(1, 1, 1));
        let b = setup(&mut world, Cell::new(1, 3), UVec3::new(1, 1, 1));
        assert!(a < b);
    }

    /*
      |   |   |
    |   |   |   |
      | B | A |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_1x1x1_vs_1x1x1_neither_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(1, 1), UVec3::new(1, 1, 1));
        let b = setup(&mut world, Cell::new(0, 1), UVec3::new(1, 1, 1));
        assert!(a.partial_cmp(&b).is_none());
    }

    /*
      |   |   |
    |   | B | A |
      |   | AB|   |
    |   | A | B |
      |   |   |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    #[should_panic]
    fn test_1x1x1_vs_1x1x1_equal() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(1, 2), UVec3::new(3, 1, 1));
        let b = setup(&mut world, Cell::new(2, 2), UVec3::new(1, 3, 1));
        let _ordering = a.partial_cmp(&b);
    }

    /*
      |   |   |
    |   |   |   |
      | B |   |   |
    |   |   |   |
      |   |   |   |
    | A2|   |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_1x1x2_vs_1x1x1_a_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(0, 4), UVec3::new(1, 1, 2));
        let b = setup(&mut world, Cell::new(0, 1), UVec3::new(1, 1, 1));
        assert!(a > b);
    }

    /*
      |   |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   | B |   |
    |   |   | A |
      |   | A |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_2x1x1_vs_1x1x1_a_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(1, 5), UVec3::new(2, 1, 1));
        let b = setup(&mut world, Cell::new(1, 3), UVec3::new(1, 1, 1));
        assert!(a > b);
    }

    /*
      |   |   |
    |   |   |   |
      |   |   |   |
    |   | A |   |
      | A | A |   |
    |   | A | A |
      | B | A | A |
    |   |   | A |
      |   |   |
    */
    #[test]
    fn test_2x4x1_vs_1x1x1_b_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(2, 6), UVec3::new(2, 4, 1));
        let b = setup(&mut world, Cell::new(0, 5), UVec3::new(1, 1, 1));
        assert!(a < b);
    }

    /*
      |   |   |
    |   |   |   |
      |   |   |   |
    |   | A |   |
      |   | A |   |
    |   |   | A |
      |   |   | A |
    |   | B2|   |
      |   |   |
    */
    #[test]
    fn test_1x4x1_vs_1x1x2_b_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(2, 5), UVec3::new(1, 4, 1));
        let b = setup(&mut world, Cell::new(1, 6), UVec3::new(1, 1, 2));
        assert!(a < b);
    }

    /*
      |   |   |
    | B2|   |   |
      | B2| A |   |
    |   | A |   |
      | A |   |   |
    |   |   |   |
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_3x1x1_vs_1x2x2_a_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(0, 3), UVec3::new(3, 1, 1));
        let b = setup(&mut world, Cell::new(0, 1), UVec3::new(1, 2, 2));
        assert!(a > b);
    }

    /*
      |   |   |
    |   |   |   |
      | A |   |   |
    | A |   |   |
      |   |   |   |
    |   |   | B2|
      |   | B2|   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_2x1x1_vs_2x1x1_neither_in_front() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(0, 2), UVec3::new(2, 1, 1));
        let b = setup(&mut world, Cell::new(1, 5), UVec3::new(2, 1, 2));
        assert!(a.partial_cmp(&b).is_none());
    }

    /*
      |   |   |
    |   |   |   |
      | A | C2|   |
    | A | A | C2|
      | A |   |   |
    |   |   | B2|
      |   |   |   |
    |   |   |   |
      |   |   |
    */
    #[test]
    fn test_abc() {
        let mut world = World::default();
        let a = setup(&mut world, Cell::new(0, 3), UVec3::new(2, 2, 1));
        let b = setup(&mut world, Cell::new(2, 4), UVec3::new(1, 1, 2));
        let c = setup(&mut world, Cell::new(2, 2), UVec3::new(1, 2, 2));
        assert!(a > c);
        assert!(b > c);
    }
}
