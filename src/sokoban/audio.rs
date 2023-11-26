use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AudioCollection>();
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
