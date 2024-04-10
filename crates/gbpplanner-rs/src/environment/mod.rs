pub mod camera;
pub mod cursor;
pub mod follow_cameras;
pub mod map;
pub mod map_generator;

use camera::CameraPlugin;
pub use camera::MainCamera;
use cursor::CursorToGroundPlugin;
pub use follow_cameras::FollowCameraMe;
use follow_cameras::FollowCamerasPlugin;
use map::MapPlugin;

use self::map_generator::GenMapPlugin;
pub use self::map_generator::TileCoordinates;
pub use map_generator::ObstacleMarker;

pub struct EnvironmentPlugin;

impl bevy::app::Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            CameraPlugin,
            FollowCamerasPlugin,
            MapPlugin,
            CursorToGroundPlugin,
            GenMapPlugin,
        ));
    }
}
