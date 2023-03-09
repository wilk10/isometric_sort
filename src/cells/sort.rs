use bevy::prelude::*;
use topological_sort::TopologicalSort;

use crate::cells::current::CurrentCells;

#[derive(Resource)]
pub struct SortThisFrame {
    pub do_sort: bool,
}

pub fn sort_items_topological(
    mut sort_this_frame: ResMut<SortThisFrame>,
    mut items: Query<(Entity, &CurrentCells, &mut Transform)>,
) {
    if !sort_this_frame.do_sort {
        return;
    }

    let mut map = TopologicalSort::<Entity>::default();

    let items_to_sort = items
        .iter()
        .filter(|(_, cells, _)| cells.dimensions.z > 0)
        .map(|(entity, cells, _)| (entity, cells.clone()))
        .collect::<Vec<(Entity, CurrentCells)>>();

    for (this_entity, this_item, _) in items.iter() {
        if this_item.dimensions.z == 0 {
            continue;
        }

        items_to_sort
            .iter()
            .filter(|(_, item)| {
                item.underneath
                    .iter()
                    .any(|under| this_item.behind.contains(under))
            })
            .for_each(|(entity_behind, _)| map.add_dependency(*entity_behind, this_entity));
    }

    for (index, entity) in map.enumerate() {
        assign_z(index, entity, items_to_sort.len(), &mut items);
    }

    sort_this_frame.do_sort = false;
}

pub fn sort_items_partial_cmp(
    mut sort_this_frame: ResMut<SortThisFrame>,
    mut items: Query<(Entity, &CurrentCells, &mut Transform)>,
) {
    if !sort_this_frame.do_sort {
        return;
    }

    let mut items_to_sort = items
        .iter()
        .filter(|(_, cells, _)| cells.dimensions.z > 0)
        .map(|(entity, cells, _)| (entity, cells.clone()))
        .collect::<Vec<(Entity, CurrentCells)>>();
    items_to_sort.sort_by(|(_, a), (_, b)| {
        a.partial_cmp(b)
            .or_else(|| a.main_cell.partial_cmp(&b.main_cell))
            .expect("Ordering must be Some")
    });

    for (index, (entity, _)) in items_to_sort.iter().enumerate() {
        assign_z(index, *entity, items_to_sort.len(), &mut items);
    }

    sort_this_frame.do_sort = false;
}

#[allow(clippy::cast_precision_loss)]
fn assign_z(
    index: usize,
    entity: Entity,
    n_items: usize,
    items: &mut Query<(Entity, &CurrentCells, &mut Transform)>,
) {
    let base_z = 0.;
    let z_span = 5.;
    let new_z = base_z + ((index as f32 / n_items as f32) * z_span);
    let (_, _, mut transform) = items.get_mut(entity).expect("Entity must exist");
    transform.translation.z = new_z;
}

#[cfg(test)]
mod sort_all_items {
    use bevy::{prelude::*, utils::FloatOrd};

    use crate::cells::{
        cell::{Cell, Direction},
        current::CurrentCells,
        sort::{sort_items_partial_cmp, sort_items_topological, SortThisFrame},
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

    fn setup<M>(
        world: &mut World,
        schedule: &mut Schedule,
        items: &[Item],
        sort_system: impl IntoSystemConfig<M>,
    ) -> Vec<Entity> {
        schedule
            .add_system(apply_system_buffers)
            .add_system(sort_system.after(apply_system_buffers));

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

    fn simple_items() -> Vec<Item> {
        vec![
            Item::new(1, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(0, Cell::new(2, 2), UVec3::new(1, 2, 2)),
            Item::new(2, Cell::new(1, 5), UVec3::new(1, 1, 2)),
        ]
    }

    fn busy_items() -> Vec<Item> {
        vec![
            Item::new(2, Cell::new(0, 3), UVec3::new(2, 2, 1)),
            Item::new(4, Cell::new(1, 6), UVec3::new(1, 2, 1)),
            Item::new(0, Cell::new(2, 1), UVec3::new(1, 1, 2)),
            Item::new(3, Cell::new(1, 5), UVec3::new(1, 1, 2)),
            Item::new(5, Cell::new(0, 6), UVec3::new(1, 1, 1)),
            Item::new(1, Cell::new(2, 3), UVec3::new(1, 3, 1)),
        ]
    }

    #[test]
    fn simple_topological() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = simple_items();
        let expected_order = setup(&mut world, &mut schedule, &items, sort_items_topological);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }

    #[test]
    fn simple_partial_cmp() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = simple_items();
        let expected_order = setup(&mut world, &mut schedule, &items, sort_items_partial_cmp);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }

    #[test]
    fn busy_topological() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = busy_items();
        let expected_order = setup(&mut world, &mut schedule, &items, sort_items_topological);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }

    #[test]
    fn busy_partial_cmp() {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        let items = busy_items();
        let expected_order = setup(&mut world, &mut schedule, &items, sort_items_partial_cmp);

        schedule.run(&mut world);

        assert_eq!(actual_order(&mut world), expected_order);
    }
}
