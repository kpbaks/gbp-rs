mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod message;
mod robot;
mod spawner;
mod variable;

pub use factorgraph::graphviz::NodeKind;
pub use factorgraph::FactorGraph;
pub use factorgraph::NodeIndex;
pub use robot::RobotId;
pub use robot::RobotState;

// use gbp_linalg::*;

// pub type Timestep = u32;

use self::robot::RobotPlugin;
use self::spawner::SpawnerPlugin;
use bevy::prelude::*;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RobotPlugin, SpawnerPlugin));
    }
}
