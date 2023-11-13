use std::ops::AddAssign;

use bevy::{ecs::system::Command, prelude::*};

use super::{cleanup::DependOnState, level::CurrentLevel, GameState};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Pause), setup)
            .add_systems(Update, handle_buttons.run_if(in_state(GameState::Pause)));
    }
}

const ALL_BUTTONS: [PauseMenuButton; 5] = [
    PauseMenuButton::Resume,
    PauseMenuButton::NextLevel,
    PauseMenuButton::PrevLevel,
    PauseMenuButton::ReturnToLevelSelect,
    PauseMenuButton::ReturnToMain,
];

#[derive(Component, Copy, Clone)]
enum PauseMenuButton {
    Resume,
    NextLevel,
    PrevLevel,
    ReturnToLevelSelect,
    ReturnToMain,
}

impl PauseMenuButton {
    pub fn name(&self) -> String {
        match self {
            PauseMenuButton::Resume => "Resume",
            PauseMenuButton::NextLevel => "Next Level",
            PauseMenuButton::PrevLevel => "Previous Level",
            PauseMenuButton::ReturnToLevelSelect => "Level Select",
            PauseMenuButton::ReturnToMain => "Main Menu",
        }
        .to_owned()
    }
}

fn setup(mut cmds: Commands) {
    cmds.add(SpawnPauseMenuButtons);
}

fn handle_buttons(
    buttons: Query<(&PauseMenuButton, &Interaction), Changed<Interaction>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    buttons.iter().for_each(|button| match button {
        (PauseMenuButton::Resume, Interaction::Pressed) => {
            game_state.set(GameState::Play);
        }
        (PauseMenuButton::NextLevel, Interaction::Pressed) => {
            current_level.add_assign(1);
            game_state.set(GameState::LevelTransition)
        }
        (PauseMenuButton::PrevLevel, Interaction::Pressed) => {
            current_level.0 = current_level.saturating_sub(1);
            game_state.set(GameState::LevelTransition)
        }
        (PauseMenuButton::ReturnToLevelSelect, Interaction::Pressed) => {
            game_state.set(GameState::LevelSelect);
        }
        (PauseMenuButton::ReturnToMain, Interaction::Pressed) => {
            game_state.set(GameState::MainMenu);
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
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
                DependOnState::single(GameState::Pause),
            ))
            .with_children(|parent| {
                for button in ALL_BUTTONS.iter() {
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
                            *button,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                button.name(),
                                TextStyle {
                                    font_size: 20.,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                }
            });
    }
}
