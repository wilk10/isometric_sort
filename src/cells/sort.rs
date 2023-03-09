use bevy::prelude::*;
use topological_sort as topo;

use crate::cells::current::CurrentCells;

#[derive(Resource)]
pub struct SortThisFrame {
    pub do_sort: bool,
}

pub fn sort_items(
    mut sort_this_frame: ResMut<SortThisFrame>,
    mut items: Query<(Entity, &CurrentCells, &mut Transform)>,
) {
    if !sort_this_frame.do_sort {
        return;
    }

    let base_z = 0.;
    let z_span = 5.;

    let mut map = topo::TopologicalSort::<Entity>::default();

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

        items_to_sort
            .iter()
            .filter(|(_, item)| {
                item.behind
                    .iter()
                    .any(|behind| this_item.underneath.contains(behind))
            })
            .for_each(|(entity_in_front, _)| map.add_dependency(this_entity, *entity_in_front));
    }

    for (index, entity) in map.enumerate() {
        let new_z = base_z + ((index as f32 / items_to_sort.len() as f32) * z_span);
        let (_, _, mut transform) = items.get_mut(entity).unwrap();
        transform.translation.z = new_z;
    }

    sort_this_frame.do_sort = false;
}
