use bevy::prelude::*;
use topological_sort::TopologicalSort;

use crate::cells::{
    current::CurrentCells,
    saved::{CompareTransforms, SortMethod},
};

pub fn sort_items_topological(mut items: Query<(Entity, &CurrentCells, &mut CompareTransforms)>) {
    let mut map = TopologicalSort::<Entity>::default();

    let n_items = items.iter().filter(|(_, cells, _)| cells.dimensions.z > 0).count();

    for (this_entity, this_item, _) in items.iter() {
        if this_item.dimensions.z == 0 {
            continue;
        }

        items
            .iter()
            .filter(|(_, cells, _)| cells.dimensions.z > 0)
            .filter(|(_, item, _)| {
                item.underneath
                    .iter()
                    .any(|under| this_item.behind.contains(under))
            })
            .for_each(|(entity_behind, _, _)| map.add_dependency(entity_behind, this_entity));
    }

    for (index, entity) in map.enumerate() {
        assign_z(
            index,
            entity,
            n_items,
            SortMethod::Topological,
            &mut items,
        );
    }
}

pub fn sort_items_partial_cmp(mut items: Query<(Entity, &CurrentCells, &mut CompareTransforms)>) {
    let mut items_to_sort = items
        .iter()
        .filter(|(_, cells, _)| cells.dimensions.z > 0)
        .map(|(entity, cells, _)| (entity, cells.clone()))
        .collect::<Vec<(Entity, CurrentCells)>>();
    items_to_sort.sort_by(|(_, a), (_, b)| b.main_cell.cmp(&a.main_cell));
    // items_to_sort.sort_by(|(_, a), (_, b)| a.prod_dims().cmp(&b.prod_dims()));
    items_to_sort.sort_by(|(_, a), (_, b)| {
        a.partial_cmp(b)
            .or_else(|| a.main_cell.partial_cmp(&b.main_cell))
            .expect("Ordering must be Some")
    });

    for (index, (entity, _)) in items_to_sort.iter().enumerate() {
        assign_z(
            index,
            *entity,
            items_to_sort.len(),
            SortMethod::PartialCmp,
            &mut items,
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn assign_z(
    index: usize,
    entity: Entity,
    n_items: usize,
    method: SortMethod,
    items: &mut Query<(Entity, &CurrentCells, &mut CompareTransforms)>,
) {
    let base_z = 0.;
    let z_span = 5.;
    let new_z = base_z + ((index as f32 / n_items as f32) * z_span);
    let (_, _, mut compare) = items.get_mut(entity).expect("Entity must exist");
    let z = compare.map.get_mut(&method).unwrap();
    *z = new_z;
}

#[cfg(test)]
mod sort_all_items {
    use bevy::{prelude::*, utils::FloatOrd};

    use crate::cells::{
        cell::{Cell, Direction},
        current::CurrentCells,
    };

    use super::*;

    #[derive(Debug)]
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

    fn setup<M>(
        world: &mut World,
        schedule: &mut Schedule,
        items: &[Item],
        sort_system: impl IntoSystemConfig<M>,
    ) -> Vec<Entity> {
        schedule
            .add_system(apply_system_buffers)
            .add_system(sort_system.after(apply_system_buffers));

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
            UVec2::new(4, 7),
        );
        world.spawn((cells, CompareTransforms::default())).id()
    }

    fn actual_order(world: &mut World, method: SortMethod) -> Vec<Entity> {
        let mut entities = world
            .query::<(Entity, &CompareTransforms)>()
            .iter(world)
            .collect::<Vec<(Entity, &CompareTransforms)>>();

        entities.sort_by(|(_, a), (_, b)| {
            FloatOrd(*a.map.get(&method).unwrap()).cmp(&FloatOrd(*b.map.get(&method).unwrap()))
        });

        entities
            .into_iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>()
    }

    fn run_simple<M>(world: &mut World, system: impl IntoSystemConfig<M>) -> Vec<Entity> {
        let mut schedule = Schedule::default();

        let items = vec![
            Item::new(1, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(0, Cell::new(2, 2), UVec3::new(1, 2, 2)),
            Item::new(2, Cell::new(1, 5), UVec3::new(1, 1, 2)),
        ];
        let expected_order = setup(world, &mut schedule, &items, system);

        schedule.run(world);

        expected_order
    }

    fn run_busy<M>(world: &mut World, system: impl IntoSystemConfig<M>) -> Vec<Entity> {
        let mut schedule = Schedule::default();

        let items = vec![
            Item::new(2, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(4, Cell::new(1, 6), UVec3::new(1, 2, 1)),
            Item::new(0, Cell::new(2, 1), UVec3::new(1, 1, 2)),
            Item::new(3, Cell::new(1, 5), UVec3::new(1, 1, 2)),
            Item::new(5, Cell::new(0, 6), UVec3::new(1, 1, 1)),
            Item::new(1, Cell::new(2, 3), UVec3::new(1, 3, 1)),
        ];
        let expected_order = setup(world, &mut schedule, &items, system);

        schedule.run(world);

        expected_order
    }

    fn run_add_later<M>(
        world: &mut World,
        system: impl IntoSystemConfig<M>,
        method: SortMethod,
    ) -> Vec<Option<usize>> {
        let mut schedule = Schedule::default();

        let initial = vec![
            Item::new(0, Cell::new(0, 4), UVec3::new(1, 1, 1)),
            Item::new(0, Cell::new(3, 3), UVec3::new(1, 1, 1)),
            Item::new(0, Cell::new(3, 4), UVec3::new(1, 1, 1)),
            Item::new(0, Cell::new(1, 5), UVec3::new(1, 1, 1)),
        ];
        let later = Item::new(0, Cell::new(2, 4), UVec3::new(2, 2, 1));
        let initial_entities = setup(world, &mut schedule, &initial, system);

        schedule.run(world);

        let later_entity = add_item(world, later.main_cell, later.dimensions);

        schedule.run(world);

        let actual = actual_order(world, method);
        let position_last_item = actual.iter().position(|entity| *entity == later_entity);
        let position_entity_index_2 = actual
            .iter()
            .position(|entity| *entity == initial_entities[2]);
        let position_entity_index_3 = actual
            .iter()
            .position(|entity| *entity == initial_entities[3]);

        vec![
            position_last_item,
            position_entity_index_2,
            position_entity_index_3,
        ]
    }

    #[test]
    fn simple_topological() {
        let mut world = World::default();
        let expected_order = run_simple(&mut world, sort_items_topological);
        assert_eq!(
            actual_order(&mut world, SortMethod::Topological),
            expected_order
        );
    }

    #[test]
    fn simple_partial_cmp() {
        let mut world = World::default();
        let expected_order = run_simple(&mut world, sort_items_partial_cmp);
        assert_eq!(
            actual_order(&mut world, SortMethod::PartialCmp),
            expected_order
        );
    }

    #[test]
    fn busy_topological() {
        let mut world = World::default();
        let expected_order = run_busy(&mut world, sort_items_topological);
        assert_eq!(
            actual_order(&mut world, SortMethod::Topological),
            expected_order
        );
    }

    #[test]
    fn busy_partial_cmp() {
        let mut world = World::default();
        let expected_order = run_busy(&mut world, sort_items_partial_cmp);
        assert_eq!(
            actual_order(&mut world, SortMethod::PartialCmp),
            expected_order
        );
    }

    #[test]
    fn add_later_topological() {
        let mut world = World::default();
        let positions = run_add_later(&mut world, sort_items_topological, SortMethod::Topological);
        let position_last_item = positions[0];
        let position_item_2 = positions[1];
        let position_item_3 = positions[2];
        assert!(position_last_item < position_item_2);
        assert!(position_last_item < position_item_3);
    }

    #[test]
    fn add_later_partial_cmp() {
        let mut world = World::default();
        let positions = run_add_later(&mut world, sort_items_partial_cmp, SortMethod::PartialCmp);
        let position_last_item = positions[0];
        let position_item_2 = positions[1];
        let position_item_3 = positions[2];
        assert!(position_last_item < position_item_2);
        assert!(position_last_item < position_item_3);
    }
}
