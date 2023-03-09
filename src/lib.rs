#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod cells;

#[cfg(test)]
mod sort_all_items {
    use bevy::{prelude::*, utils::FloatOrd};

    use crate::cells::{
        cell::{Cell, Direction},
        current::CurrentCells,
        sort::{sort_items, SortThisFrame},
    };

    struct Item {
        expected_index: usize,
        main_cell: Cell,
        dimensions: UVec3,
    }

    impl Item {
        fn new(expected_index: usize, main_cell: Cell, dimensions: UVec3) -> Self {
            Self {
                expected_index,
                main_cell,
                dimensions,
            }
        }
    }

    fn setup(world: &mut World, schedule: &mut Schedule, items: &[Item]) -> Vec<Entity> {
        schedule
            .add_system(apply_system_buffers)
            .add_system(sort_items.after(apply_system_buffers));

        world.insert_resource(SortThisFrame { do_sort: true });

        let mut expected = items
            .iter()
            .map(|item| {
                (
                    item.expected_index,
                    add_item(world, item.main_cell, item.dimensions),
                )
            })
            .collect::<Vec<(usize, Entity)>>();
        expected.sort_by(|(a, _), (b, _)| a.cmp(&b));
        expected
            .into_iter()
            .map(|(_, entity)| entity)
            .collect::<Vec<Entity>>()
    }

    fn add_item(world: &mut World, main_cell: Cell, dimensions: UVec3) -> Entity {
        let cells = CurrentCells::new(
            main_cell,
            dimensions,
            Direction::BottomRight,
            UVec2::new(3, 7),
        );
        world
            .spawn((cells, Transform::from_translation(Vec3::ZERO)))
            .id()
    }

    fn actual_order(world: &mut World) -> Vec<Entity> {
        let mut entities = world
            .query::<(Entity, &Transform)>()
            .iter(world)
            .collect::<Vec<(Entity, &Transform)>>();
        entities
            .sort_by(|(_, a), (_, b)| FloatOrd(a.translation.z).cmp(&FloatOrd(b.translation.z)));
        entities
            .into_iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>()
    }

    #[test]
    fn simple() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = vec![
            Item::new(1, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(0, Cell::new(2, 2), UVec3::new(1, 2, 2)),
            Item::new(2, Cell::new(1, 5), UVec3::new(1, 1, 2)),
        ];
        let expected_order = setup(&mut world, &mut schedule, &items);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }

    #[test]
    fn busy() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = vec![
            Item::new(2, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(4, Cell::new(1, 6), UVec3::new(1, 2, 1)),
            Item::new(0, Cell::new(2, 1), UVec3::new(1, 1, 2)),
            Item::new(3, Cell::new(1, 5), UVec3::new(1, 1, 2)),
            Item::new(5, Cell::new(0, 6), UVec3::new(1, 1, 1)),
            Item::new(1, Cell::new(2, 3), UVec3::new(1, 3, 1)),
        ];
        let expected_order = setup(&mut world, &mut schedule, &items);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }
}
