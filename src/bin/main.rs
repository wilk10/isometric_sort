use bevy::{app::AppExit, prelude::*};
use isometric_sort::cells::{
    cell::{Cell, Direction},
    current::CurrentCells,
    saved::{Check, CompareTransforms, Corrects, EntitiesNearby, Results, SavedCells, SortMethod},
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
        .init_resource::<Results>()
        .add_startup_system(load_scene)
        .add_startup_system(load_checks)
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
        .add_system(
            print_results
                .after(check_z)
                .run_if(in_state(TestState::Compare)),
        )
        .add_system(
            exit.after(print_results)
                .run_if(in_state(TestState::Compare)),
        )
        .run();
}

const SCENE_ID: u8 = 1;

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicSceneBundle {
        scene: asset_server.load(format!("scenes/{SCENE_ID:?}/debug_scene.scn.ron")),
        ..default()
    });
}

fn load_checks(mut commands: Commands, asset_server: Res<AssetServer>) {
    if let Ok(handles) = asset_server.load_folder(format!("scenes/{SCENE_ID:?}/checks")) {
        for untyped_handle in &handles {
            let handle = untyped_handle.clone().typed();
            commands.spawn((DynamicSceneBundle {
                scene: handle,
                ..default()
            },));
        }
    }
}

// TODO: find a better way to compare them than checking if they have Transform or not
fn map_saved_cells_to_current(
    mut commands: Commands,
    mut state: ResMut<NextState<TestState>>,
    items: Query<(Entity, &SavedCells), With<Transform>>,
    checks: Query<(Entity, &SavedCells), Without<Transform>>,
) {
    if items.iter().count() == 0 || checks.iter().count() == 0 {
        return;
    }
    dbg!(items.iter().count());
    dbg!(checks.iter().count());

    for (entity, saved) in items.iter() {
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
    for (entity, saved) in checks.iter() {
        let current = CurrentCells::new(
            saved.main_cell,
            saved.dimensions,
            saved.facing,
            UVec2::new(128, 128),
        );
        commands
            .entity(entity)
            .remove::<SavedCells>()
            .insert((current, Check));
    }

    state.set(TestState::Compare);
}

fn find_nearby_entities(
    mut commands: Commands,
    items: Query<(Entity, &CurrentCells), Without<Check>>,
    checks: Query<(Entity, &CurrentCells), With<Check>>,
) {
    for (check_entity, check_cells) in checks.iter() {
        let (corresponding_entity, _) = items
            .iter()
            .find(|(_, cells)| cells.main_cell == check_cells.main_cell)
            .unwrap();

        let entities_behind = items
            .iter()
            .filter(|(_, cells)| {
                cells
                    .underneath
                    .iter()
                    .any(|under| check_cells.behind.contains(under))
            })
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();

        let entities_in_front = items
            .iter()
            .filter(|(_, cells)| {
                cells
                    .behind
                    .iter()
                    .any(|behind| check_cells.underneath.contains(behind))
            })
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();

        let entities_nearby = EntitiesNearby {
            corresponding: corresponding_entity,
            behind: entities_behind,
            in_front: entities_in_front,
        };
        commands.entity(check_entity).insert(entities_nearby);
    }
}

fn check_z(
    mut results: ResMut<Results>,
    items: Query<&CompareTransforms>,
    checks: Query<&EntitiesNearby>,
) {
    for check in checks.iter() {
        SortMethod::all()
            .iter()
            .map(|method| {
                let corresponding_z = items
                    .get(check.corresponding)
                    .ok()
                    .and_then(|compare| compare.map.get(method))
                    .unwrap();
                let behind_zs = check
                    .behind
                    .iter()
                    .flat_map(|entity| items.get(*entity).ok())
                    .flat_map(|compare| compare.map.get(method))
                    .collect::<Vec<&f32>>();
                let in_front_zs = check
                    .in_front
                    .iter()
                    .flat_map(|entity| items.get(*entity).ok())
                    .flat_map(|compare| compare.map.get(method))
                    .collect::<Vec<&f32>>();
                (method, corresponding_z, behind_zs, in_front_zs)
            })
            .for_each(|(method, item_z, behind_z, in_front_z)| {
                println!("======");
                dbg!(&method);
                dbg!(&behind_z);
                dbg!(&item_z);
                dbg!(&in_front_z);
                let are_behind_z_correct = behind_z.into_iter().all(|z| z < item_z);
                let are_in_front_z_correct = in_front_z.into_iter().all(|z| z > item_z);
                dbg!(are_behind_z_correct);
                dbg!(are_in_front_z_correct);

                let corrects = results.map.get_mut(method).unwrap();
                corrects.push(Corrects {
                    all_behind: are_behind_z_correct,
                    all_in_front: are_in_front_z_correct,
                });
            });
    }
}

fn print_results(results: Res<Results>) {
    for method in &SortMethod::all() {
        println!("======================");
        let corrects = results.map.get(method).unwrap();
        dbg!(method);
        corrects.iter().for_each(|corrects| {
            dbg!(corrects.are_both_true());
        })
    }
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
