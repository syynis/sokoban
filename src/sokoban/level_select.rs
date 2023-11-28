use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;
use leafwing_input_manager::prelude::ActionState;

use super::{
    cleanup::DependOnState,
    level::{LevelCollection, Levels},
    AssetsCollection, GameState, SokobanActions,
};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .register_type::<CurrentLevel>()
            .add_systems(
                OnEnter(GameState::LevelSelect),
                (spawn_level_select, apply_deferred).chain(),
            )
            .add_systems(
                Update,
                (handle_buttons, ui_navigation, render_selected_border)
                    .run_if(in_state(GameState::LevelSelect)),
            );
    }
}

#[derive(Resource, Deref, DerefMut, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct CurrentLevel(pub usize);

#[derive(Component, Deref, DerefMut)]
struct LevelButton(pub usize);

fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Query<(&LevelButton, &Interaction), Changed<Interaction>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    buttons
        .iter()
        .for_each(|(level, interaction)| match interaction {
            Interaction::Pressed => {
                current_level.0 = **level;
                game_state.set(GameState::LevelTransition);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        });
}

fn ui_navigation(
    level_collection: Res<LevelCollection>,
    levels: Res<Assets<Levels>>,
    mut current_level: ResMut<CurrentLevel>,
    navigation_actions: Query<&ActionState<SokobanActions>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(navigation_actions) = navigation_actions.get_single() else {
        return;
    };
    let amount_levels = levels
        .get(&level_collection.levels)
        .expect("Level assets should be loaded")
        .len();
    let cols = 5;

    let mut current = **current_level;

    if navigation_actions.just_pressed(SokobanActions::UiNavUp) {
        current = (current + amount_levels - cols) % amount_levels;
    }
    if navigation_actions.just_pressed(SokobanActions::UiNavRight) {
        current = (current + 1) % amount_levels;
    }
    if navigation_actions.just_pressed(SokobanActions::UiNavDown) {
        current = (current + cols) % amount_levels;
    }
    if navigation_actions.just_pressed(SokobanActions::UiNavLeft) {
        current = (current + amount_levels - 1) % amount_levels;
    }
    current_level.0 = current;

    if navigation_actions.just_pressed(SokobanActions::UiNavSelect) {
        game_state.set(GameState::LevelTransition);
    }
}

fn render_selected_border(
    current_level: Res<CurrentLevel>,
    mut buttons: Query<(&LevelButton, &mut BorderColor)>,
) {
    for (button, mut border_color) in buttons.iter_mut() {
        if **button == **current_level {
            *border_color = BorderColor(Color::RED);
        } else {
            *border_color = BorderColor(Color::NONE);
        }
    }
}

fn spawn_level_select(
    mut cmds: Commands,
    level_assets: Res<LevelCollection>,
    levels: Res<Assets<Levels>>,
    assets: Res<AssetsCollection>,
) {
    let button_texture = assets.button.clone_weak();
    let button_style = Style {
        width: Val::Px(75.0),
        height: Val::Px(75.0),
        margin: UiRect::all(Val::Px(10.)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.)),
        ..default()
    };
    let amount_levels = levels
        .get(&level_assets.levels)
        .expect("Level assets should be loaded")
        .len();
    let cols = 5;
    let rows = (amount_levels / cols) + 1;

    let mut children = Vec::new();
    for r in 0..rows {
        let mut buttons = Vec::new();
        for c in 0..cols {
            let idx = c + r * cols;
            if idx >= amount_levels {
                continue;
            }
            let id = cmds
                .spawn((
                    NodeBundle {
                        style: button_style.clone(),
                        focus_policy: bevy::ui::FocusPolicy::Block,
                        ..default()
                    },
                    Interaction::default(),
                    NineSliceTexture::new(button_texture.clone_weak()),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("{}", idx + 1),
                        TextStyle {
                            font_size: 20.,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                })
                .id();

            cmds.entity(id).insert(LevelButton(idx));
            buttons.push(id);
        }
        let row_node = cmds
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                ..default()
            })
            .push_children(&buttons)
            .id();
        children.push(row_node);
    }
    cmds.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            ..default()
        },
        DependOnState::single(GameState::LevelSelect),
    ))
    .push_children(&children);
}
