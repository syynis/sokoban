use std::ops::AddAssign;

use bevy::{ecs::system::Command, prelude::*};
use leafwing_input_manager::prelude::ActionState;

use super::{cleanup::DependOnState, level::CurrentLevel, GameState, SokobanActions};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Pause), setup)
            .add_systems(
                Update,
                (handle_buttons, unpause).run_if(in_state(GameState::Pause)),
            );
    }
}

#[derive(Component)]
enum PauseMenuButton {
    ReturnToMain,
    NextLevel,
    PrevLevel,
}

fn setup(mut cmds: Commands) {
    cmds.add(SpawnPauseMenuButtons);
}
fn unpause(
    actions: Query<&ActionState<SokobanActions>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(actions) = actions.get_single() else {
        return;
    };
    if actions.just_pressed(SokobanActions::Pause) {
        game_state.set(GameState::Play)
    }
}

fn handle_buttons(
    buttons: Query<(&PauseMenuButton, &Interaction), Changed<Interaction>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    buttons.iter().for_each(|button| match button {
        (PauseMenuButton::ReturnToMain, Interaction::Pressed) => {
            game_state.set(GameState::MainMenu);
        }
        (PauseMenuButton::NextLevel, Interaction::Pressed) => {
            current_level.add_assign(1);
            game_state.set(GameState::LevelTransition)
        }
        (PauseMenuButton::PrevLevel, Interaction::Pressed) => {
            current_level.0 = current_level.saturating_sub(1);
            game_state.set(GameState::LevelTransition)
        }
        _ => {}
    });
}

pub struct SpawnPauseMenuButtons;

impl Command for SpawnPauseMenuButtons {
    fn apply(self, world: &mut World) {
        world
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        align_content: AlignContent::Center,
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::ColumnReverse,
                        ..default()
                    },
                    ..default()
                },
                DependOnState::single(GameState::Pause),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(150.0),
                                height: Val::Px(65.0),
                                margin: UiRect {
                                    top: Val::Px(10.),
                                    bottom: Val::Px(10.),
                                    ..default()
                                },
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::BLACK),
                            focus_policy: bevy::ui::FocusPolicy::Block,
                            ..default()
                        },
                        PauseMenuButton::ReturnToMain,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Return to Main Menu",
                            TextStyle {
                                font_size: 20.,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });

                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(150.0),
                                height: Val::Px(65.0),
                                margin: UiRect {
                                    top: Val::Px(10.),
                                    bottom: Val::Px(10.),
                                    ..default()
                                },
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::BLACK),
                            focus_policy: bevy::ui::FocusPolicy::Block,
                            ..default()
                        },
                        PauseMenuButton::NextLevel,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Next Level",
                            TextStyle {
                                font_size: 20.,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });

                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(150.0),
                                height: Val::Px(65.0),
                                margin: UiRect {
                                    top: Val::Px(10.),
                                    bottom: Val::Px(10.),
                                    ..default()
                                },
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::BLACK),
                            focus_policy: bevy::ui::FocusPolicy::Block,
                            ..default()
                        },
                        PauseMenuButton::PrevLevel,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Previous Level",
                            TextStyle {
                                font_size: 20.,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            });
    }
}
