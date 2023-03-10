use bevy::{app::AppExit, prelude::*};
use isometric_sort::cells::{
    cell::{Cell, Direction},
    current::CurrentCells,
    saved::{CompareTransforms, EntitiesNearby, Mistake, SavedCells},
    sort::{sort_items_partial_cmp, sort_items_topological},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
enum TestState {
    #[default]
    Prepare,
    Compare,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_state::<TestState>()
        .register_type::<Cell>()
        .register_type::<Direction>()
        .register_type::<SavedCells>()
        .add_startup_system(load_scene)
        .add_startup_system(load_mistakes)
        .add_system(map_saved_cells_to_current)
        .add_systems(
            (
                find_nearby_entities,
                sort_items_topological,
                sort_items_partial_cmp,
            )
                .in_schedule(OnEnter(TestState::Compare)),
        )
        .add_system(check_z.run_if(in_state(TestState::Compare)))
        .add_system(exit.run_if(in_state(TestState::Compare)))
        .run();
}

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicSceneBundle {
        scene: asset_server.load("scenes/debug_scene.scn.ron"),
        ..default()
    });
}

fn load_mistakes(mut commands: Commands, asset_server: Res<AssetServer>) {
    if let Ok(handles) = asset_server.load_folder("scenes/mistakes") {
        for untyped_handle in &handles {
            let handle = untyped_handle.clone().typed();
            commands.spawn((DynamicSceneBundle {
                scene: handle,
                ..default()
            },));
        }
    }
}

fn map_saved_cells_to_current(
    mut commands: Commands,
    mut state: ResMut<NextState<TestState>>,
    items: Query<(Entity, &SavedCells), With<Transform>>,
    mistakes: Query<(Entity, &SavedCells), Without<Transform>>,
) {
    if items.iter().count() == 0 || mistakes.iter().count() == 0 {
        return;
    }
    dbg!(items.iter().count());
    dbg!(mistakes.iter().count());

    for (entity, saved) in items.iter() {
        // should i do this? it may mess up with the ground truth around mistakes
        if saved.dimensions.z == 0 {
            commands.entity(entity).despawn();
        } else {
            let current = CurrentCells::new(
                saved.main_cell,
                saved.dimensions,
                saved.facing,
                UVec2::new(128, 128),
            );
            commands
                .entity(entity)
                .remove::<SavedCells>()
                .insert((current, CompareTransforms::default()));
        }
    }
    for (entity, saved) in mistakes.iter() {
        let current = CurrentCells::new(
            saved.main_cell,
            saved.dimensions,
            saved.facing,
            UVec2::new(128, 128),
        );
        commands
            .entity(entity)
            .remove::<SavedCells>()
            .insert((current, Mistake));
    }

    state.set(TestState::Compare);
}

fn find_nearby_entities(
    mut commands: Commands,
    items: Query<(Entity, &CurrentCells), Without<Mistake>>,
    mistakes: Query<(Entity, &CurrentCells), With<Mistake>>,
) {
    for (mistake_entity, mistake_cells) in mistakes.iter() {
        let (identical_entity, _) = items
            .iter()
            .find(|(_, cells)| cells.main_cell == mistake_cells.main_cell)
            .unwrap();
        let entities_behind = items
            .iter()
            .filter(|(_, cells)| {
                cells
                    .underneath
                    .iter()
                    .any(|under| mistake_cells.behind.contains(under))
            })
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();
        let entities_in_front = items
            .iter()
            .filter(|(_, cells)| {
                cells
                    .behind
                    .iter()
                    .any(|behind| mistake_cells.underneath.contains(behind))
            })
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();
        let entities_nearby = EntitiesNearby {
            identical: identical_entity,
            behind: entities_behind,
            in_front: entities_in_front,
        };
        dbg!(&entities_nearby);
        commands.entity(mistake_entity).insert(entities_nearby);
    }
}

fn check_z(compares: Query<&CompareTransforms>) {
    for _compare in compares.iter() {
        // TODO
    }
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
