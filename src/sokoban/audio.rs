use bevy::{audio::*, prelude::*};
use bevy_asset_loader::prelude::AssetCollection;

use super::{GameState, SokobanEvent};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AudioCollection>()
            .register_type::<VolumeSettings>()
            .init_resource::<VolumeSettings>()
            .add_systems(
                Update,
                handle_audio
                    .run_if(in_state(GameState::Play))
                    .run_if(on_event::<SokobanEvent>()),
            );
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VolumeSettings {
    pub sfx_vol: f32,
    pub music_vol: f32,
    increment: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            music_vol: 0.4,
            sfx_vol: 0.4,
            increment: 0.2,
        }
    }
}

impl VolumeSettings {
    pub fn change_sfx_vol(&mut self) {
        self.sfx_vol = (self.sfx_vol + self.increment).clamp(0., 1.);
    }
    pub fn change_music_vol(&mut self) {
        self.music_vol = (self.music_vol + self.increment).clamp(0., 1.);
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
    volume_settings: Res<VolumeSettings>,
) {
    let settings = PlaybackSettings {
        mode: PlaybackMode::Despawn,
        volume: Volume::Absolute(VolumeLevel::new(volume_settings.sfx_vol)),
        ..default()
    };
    for ev in sokoban_events.read() {
        match ev {
            SokobanEvent::PlayerMoved => cmds.spawn(AudioBundle {
                source: audio.walk.clone(),
                settings,
            }),
            SokobanEvent::PlayerPush => cmds.spawn(AudioBundle {
                source: audio.push_player.clone(),
                settings,
            }),
            SokobanEvent::BallPush => todo!(),
            SokobanEvent::BallHitWall => todo!(),
            SokobanEvent::EntityInVoid => todo!(),
        };
    }
}
