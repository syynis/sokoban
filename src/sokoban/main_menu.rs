use bevy::{app::AppExit, ecs::system::Command, prelude::*};

use super::{cleanup::DependOnState, GameState};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            (spawn_main_menu, apply_deferred).chain(),
        )
        .add_systems(Update, handle_buttons.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
enum MainMenuButton {
    Play,
    Exit,
}

fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Query<(&MainMenuButton, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<AppExit>,
) {
    buttons.iter().for_each(|button| match button {
        (MainMenuButton::Play, Interaction::Pressed) => game_state.set(GameState::LevelSelect),
        (MainMenuButton::Exit, Interaction::Pressed) => events.send(AppExit),
        _ => {}
    });
}

fn spawn_main_menu(mut cmds: Commands) {
    cmds.add(SpawnMainMenuButtons);
}

pub struct SpawnMainMenuButtons;

impl Command for SpawnMainMenuButtons {
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
                DependOnState::single(GameState::MainMenu),
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
                        MainMenuButton::Play,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Play",
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
                        MainMenuButton::Exit,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Exit",
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
