use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;

use super::{GameState, SokobanEvent};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AudioCollection>()
            .add_systems(Update, handle_audio
                .run_if(in_state(GameState::Play)).run_if(on_event::<SokobanEvent>())
            );
    }
}

#[derive(Resource, AssetCollection, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct AudioCollection {
    #[asset(path = "walk.wav")]
    pub walk: Handle<AudioSource>,
    #[asset(path = "player_ball.wav")]
    pub push_player: Handle<AudioSource>,
    #[asset(path = "ball_ball.wav")]
    pub push_ball: Handle<AudioSource>,
    #[asset(path = "ball_wall.wav")]
    pub wall: Handle<AudioSource>,
    #[asset(path = "void.wav")]
    pub void: Handle<AudioSource>,
}

fn handle_audio(
    mut cmds: Commands,
    mut sokoban_events: EventReader<SokobanEvent>,
    audio: Res<AudioCollection>,
) {
    for ev in sokoban_events.read() {
        match ev {
            SokobanEvent::PlayerMoved => cmds.spawn(AudioBundle {
                source: audio.walk.clone(),
                settings: PlaybackSettings::DESPAWN,
            }),
            SokobanEvent::PlayerPush => cmds.spawn(AudioBundle {
                source: audio.push_player.clone(),
                settings: PlaybackSettings::DESPAWN,
            }),
            SokobanEvent::BallPush => todo!(),
            SokobanEvent::BallHitWall => todo!(),
            SokobanEvent::EntityInVoid => todo!(),
        };
    }
}
