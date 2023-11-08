use bevy::{ecs::system::Command, prelude::*};

use super::{cleanup::DependOnState, level::CurrentLevel, GameState};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelSelect), spawn_level_select)
            .add_systems(
                Update,
                handle_buttons.run_if(in_state(GameState::LevelSelect)),
            );
    }
}

#[derive(Component, Deref, DerefMut)]
struct LevelButton(pub usize);

fn spawn_level_select(mut cmds: Commands) {
    cmds.add(SpawnLevelSelectButtons);
}

pub struct SpawnLevelSelectButtons;

impl Command for SpawnLevelSelectButtons {
    fn apply(self, world: &mut World) {
        let rows = 4;
        let cols = 4;

        let mut children = Vec::new();
        for r in 0..rows {
            let mut buttons = Vec::new();
            for c in 0..cols {
                let id = world
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(75.0),
                                height: Val::Px(75.0),
                                margin: UiRect::all(Val::Px(10.)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::DARK_GRAY),
                            focus_policy: bevy::ui::FocusPolicy::Block,
                            ..default()
                        },
                        LevelButton(c + r * cols),
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            format!("{}", c + r * cols),
                            TextStyle {
                                font_size: 20.,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    })
                    .id();
                buttons.push(id);
            }
            let row_node = world
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
        world
            .spawn((
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
                DependOnState(GameState::LevelSelect),
            ))
            .push_children(&children);
    }
}

fn handle_buttons(
    mut cmds: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Query<(&LevelButton, &Interaction), Changed<Interaction>>,
) {
    buttons
        .iter()
        .for_each(|(level, interaction)| match interaction {
            Interaction::Pressed => {
                cmds.insert_resource(CurrentLevel(**level));
                game_state.set(GameState::Play);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        });
}
