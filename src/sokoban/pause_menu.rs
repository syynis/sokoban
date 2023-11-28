use std::ops::AddAssign;

use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;
use leafwing_input_manager::prelude::ActionState;

use super::{
    cleanup::DependOnState, level_select::CurrentLevel, AssetsCollection, GameState, SokobanActions,
};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionEvent>()
            .init_resource::<SelectedButton>()
            .register_type::<SelectedButton>()
            .add_systems(OnEnter(GameState::Pause), setup)
            .add_systems(
                Update,
                (
                    handle_buttons.before(handle_interaction),
                    ui_navigation.before(handle_interaction),
                    handle_interaction,
                    render_selected_border,
                )
                    .run_if(in_state(GameState::Pause)),
            );
    }
}

const ALL_BUTTONS: [PauseMenuButton; 5] = [
    PauseMenuButton::Resume,
    PauseMenuButton::NextLevel,
    PauseMenuButton::PrevLevel,
    PauseMenuButton::ReturnToLevelSelect,
    PauseMenuButton::ReturnToMain,
];

#[derive(Component, Copy, Clone, PartialEq)]
enum PauseMenuButton {
    Resume,
    NextLevel,
    PrevLevel,
    ReturnToLevelSelect,
    ReturnToMain,
}

impl From<PauseMenuButton> for GameState {
    fn from(value: PauseMenuButton) -> Self {
        match value {
            PauseMenuButton::Resume => GameState::Play,
            PauseMenuButton::NextLevel => GameState::LevelTransition,
            PauseMenuButton::PrevLevel => GameState::LevelTransition,
            PauseMenuButton::ReturnToLevelSelect => GameState::LevelSelect,
            PauseMenuButton::ReturnToMain => GameState::MainMenu,
        }
    }
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

fn setup(mut cmds: Commands, assets: Res<AssetsCollection>) {
    let button_texture = assets.button.clone_weak();
    cmds.spawn((
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
                    NodeBundle {
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
                        focus_policy: bevy::ui::FocusPolicy::Block,
                        ..default()
                    },
                    Interaction::default(),
                    NineSliceTexture::new(button_texture.clone_weak()),
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

#[derive(Event, Deref, DerefMut)]
struct InteractionEvent(pub PauseMenuButton);

fn handle_interaction(
    mut events: EventReader<InteractionEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    for ev in events.read() {
        match **ev {
            PauseMenuButton::NextLevel => {
                current_level.add_assign(1);
            }
            PauseMenuButton::PrevLevel => {
                current_level.0 = current_level.saturating_sub(1);
            }
            _ => {}
        }
        game_state.set(GameState::from(**ev));
    }
}

fn handle_buttons(
    buttons: Query<(&PauseMenuButton, &Interaction), Changed<Interaction>>,
    mut event_writer: EventWriter<InteractionEvent>,
) {
    buttons.iter().for_each(|button| {
        if let (button, Interaction::Pressed) = button {
            event_writer.send(InteractionEvent(*button));
        }
    });
}

#[derive(Resource, Deref, DerefMut, Default, Reflect)]
#[reflect(Resource)]
struct SelectedButton(pub usize);

fn ui_navigation(
    navigation_actions: Query<&ActionState<SokobanActions>>,
    mut selected_button: ResMut<SelectedButton>,
    mut event_writer: EventWriter<InteractionEvent>,
) {
    let Ok(navigation_actions) = navigation_actions.get_single() else {
        return;
    };
    if navigation_actions.just_pressed(SokobanActions::UiNavUp) {
        selected_button.0 = (**selected_button + ALL_BUTTONS.len() - 1) % ALL_BUTTONS.len();
    }
    if navigation_actions.just_pressed(SokobanActions::UiNavDown) {
        selected_button.0 = (**selected_button + 1) % ALL_BUTTONS.len();
    }
    if navigation_actions.just_pressed(SokobanActions::UiNavSelect) {
        event_writer.send(InteractionEvent(ALL_BUTTONS[**selected_button]));
    }
}

fn render_selected_border(
    selected_button: Res<SelectedButton>,
    mut buttons: Query<(&PauseMenuButton, &mut BorderColor)>,
) {
    for (button, mut border_color) in buttons.iter_mut() {
        if *button == ALL_BUTTONS[**selected_button] {
            *border_color = BorderColor(Color::RED);
        } else {
            *border_color = BorderColor(Color::NONE);
        }
    }
}
