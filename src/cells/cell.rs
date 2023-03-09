use bevy::math::{IVec2, UVec2};
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
}

impl Cell {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn next_cell(self, direction: Direction, map_size: UVec2) -> Option<Cell> {
        self.nth_cell_in_direction(direction, 1, map_size)
    }

    fn maybe_new_from_offset(cell: IVec2, map_max: IVec2) -> Option<Self> {
        let respects_lower_map_bound = cell.x >= 0 && cell.y >= 0;
        let respects_higher_map_bound = cell.x < map_max.x && cell.y < map_max.y;

        (respects_lower_map_bound && respects_higher_map_bound).then(|| {
            let offset_cell = cell.as_uvec2();
            Cell::new(offset_cell.x, offset_cell.y)
        })
    }

    #[allow(dead_code)]
    fn all_next_cells(self, map_size: UVec2) -> impl Iterator<Item = Cell> {
        self.directional_next_cells(map_size, Direction::all().iter())
            .into_iter()
            .flatten()
    }

    #[allow(dead_code)]
    fn diagonal_next_cells(self, map_size: UVec2) -> impl Iterator<Item = Cell> {
        self.directional_next_cells(map_size, Direction::diagonals().iter())
            .into_iter()
            .flatten()
    }

    fn directional_next_cells<'d>(
        self,
        map_size: UVec2,
        directions: impl Iterator<Item = &'d Direction>,
    ) -> Vec<Option<Cell>> {
        directions.fold(Vec::new(), |mut cells, direction| {
            let next_cell = self.next_cell(*direction, map_size);
            cells.push(next_cell);
            cells
        })
    }

    fn nth_cell_in_direction(self, direction: Direction, n: u32, map_size: UVec2) -> Option<Cell> {
        let map_max = map_size.as_ivec2();
        (0..n).fold(Some(self), |mut next_cell, _| {
            next_cell = next_cell
                .map(|cell| {
                    let mut next_cell = IVec2::from(cell);
                    next_cell += cell.offset(direction);
                    next_cell
                })
                .and_then(|cell| Self::maybe_new_from_offset(cell, map_max));
            next_cell
        })
    }

    #[allow(clippy::match_same_arms)]
    fn offset(self, direction: Direction) -> IVec2 {
        let is_y_even = self.y % 2 == 0;
        match (direction, is_y_even) {
            (Direction::Top, _) => IVec2::new(0, -2),
            (Direction::TopRight, true) => IVec2::new(0, -1),
            (Direction::TopRight, false) => IVec2::new(1, -1),
            (Direction::Right, _) => IVec2::new(1, 0),
            (Direction::BottomRight, true) => IVec2::new(0, 1),
            (Direction::BottomRight, false) => IVec2::new(1, 1),
            (Direction::Bottom, _) => IVec2::new(0, 2),
            (Direction::BottomLeft, true) => IVec2::new(-1, 1),
            (Direction::BottomLeft, false) => IVec2::new(0, 1),
            (Direction::Left, _) => IVec2::new(-1, 0),
            (Direction::TopLeft, true) => IVec2::new(-1, -1),
            (Direction::TopLeft, false) => IVec2::new(0, -1),
        }
    }
}

impl From<Cell> for IVec2 {
    fn from(cell: Cell) -> Self {
        UVec2::new(cell.x, cell.y).as_ivec2()
    }
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        // let y_order = self.y.cmp(&other.y);
        // if y_order == Ordering::Equal {
        //     self.x.cmp(&other.x)
        // } else {
        //     y_order
        // }
        self.y.cmp(&other.y)
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell(x: {}, y: {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

impl Direction {
    #[allow(dead_code)]
    fn all() -> [Self; 8] {
        [
            Self::Top,
            Self::TopRight,
            Self::Right,
            Self::BottomRight,
            Self::Bottom,
            Self::BottomLeft,
            Self::Left,
            Self::TopLeft,
        ]
    }

    #[allow(dead_code)]
    fn diagonals() -> [Self; 4] {
        [
            Self::TopRight,
            Self::BottomRight,
            Self::BottomLeft,
            Self::TopLeft,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_next_cells_01() {
        let cell = Cell::new(0, 1);
        let expected = vec![
            Cell::new(1, 0),
            Cell::new(1, 1),
            Cell::new(1, 2),
            Cell::new(0, 3),
            Cell::new(0, 2),
            Cell::new(0, 0),
        ];
        let actual = cell.all_next_cells(UVec2::new(4, 6)).collect::<Vec<Cell>>();
        assert_eq!(actual, expected)
    }

    #[test]
    fn find_next_cells_13() {
        let cell = Cell::new(1, 3);
        let expected = vec![
            Cell::new(1, 1),
            Cell::new(2, 2),
            Cell::new(2, 3),
            Cell::new(2, 4),
            Cell::new(1, 5),
            Cell::new(1, 4),
            Cell::new(0, 3),
            Cell::new(1, 2),
        ];
        let actual = cell.all_next_cells(UVec2::new(4, 6)).collect::<Vec<Cell>>();
        assert_eq!(actual, expected)
    }

    #[test]
    fn find_next_cells_22() {
        let cell = Cell::new(2, 2);
        let expected = vec![
            Cell::new(2, 0),
            Cell::new(2, 1),
            Cell::new(3, 2),
            Cell::new(2, 3),
            Cell::new(2, 4),
            Cell::new(1, 3),
            Cell::new(1, 2),
            Cell::new(1, 1),
        ];
        let actual = cell.all_next_cells(UVec2::new(4, 6)).collect::<Vec<Cell>>();
        assert_eq!(actual, expected)
    }

    #[test]
    fn find_next_cells_34() {
        let cell = Cell::new(3, 4);
        let expected = vec![
            Cell::new(3, 2),
            Cell::new(3, 3),
            Cell::new(3, 5),
            Cell::new(2, 5),
            Cell::new(2, 4),
            Cell::new(2, 3),
        ];
        let actual = cell.all_next_cells(UVec2::new(4, 6)).collect::<Vec<Cell>>();
        assert_eq!(actual, expected)
    }
}
