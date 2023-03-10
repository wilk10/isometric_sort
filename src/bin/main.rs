use bevy::{app::AppExit, prelude::*};
use isometric_sort::cells::{
    cell::{Cell, Direction},
    current::CurrentCells,
    saved::SavedCells,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
enum TestState {
    #[default]
    Prepare,
    Compare,
}

#[derive(Component)]
struct Mistake;

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
        .add_system(compare.in_schedule(OnEnter(TestState::Compare)))
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
    entities: Query<(Entity, &SavedCells), With<Transform>>,
    mistakes: Query<(Entity, &SavedCells), Without<Transform>>,
) {
    if entities.iter().count() == 0 || mistakes.iter().count() == 0 {
        return;
    }
    dbg!(entities.iter().count());
    dbg!(mistakes.iter().count());

    for (entity, saved) in entities.iter() {
        let current = CurrentCells::new(
            saved.main_cell,
            saved.dimensions,
            saved.facing,
            UVec2::new(128, 128),
        );
        commands
            .entity(entity)
            .remove::<SavedCells>()
            .insert(current);
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

fn compare(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
