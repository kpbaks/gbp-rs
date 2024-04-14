use super::factor::ExternalVariableId;

pub struct Node {
    pub index: usize,
    pub kind:  NodeKind,
}

impl Node {
    pub fn color(&self) -> &'static str {
        self.kind.color()
    }

    pub fn shape(&self) -> &'static str {
        self.kind.shape()
    }

    pub fn width(&self) -> f64 {
        self.kind.width()
    }
}

pub enum NodeKind {
    Variable { x: f32, y: f32 },
    InterRobotFactor(ExternalVariableId),
    // InterRobotFactor {
    //     /// The id of the robot the interrobot factor is connected to
    //     other_robot_id: RobotId,
    //     /// The index of the variable in the other robots factorgraph, that the interrobot
    // factor is connected with     variable_index_in_other_robot: usize,
    // },
    DynamicFactor,
    ObstacleFactor,
    PoseFactor,
}

impl NodeKind {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Variable { .. } => "#eff1f5",         // latte base (white)
            Self::InterRobotFactor { .. } => "#a6da95", // green
            Self::DynamicFactor => "#8aadf4",           // blue
            // Self::ObstacleFactor => "#c6a0f6",          // mauve (purple)
            Self::ObstacleFactor => "#ee99a0", // mauve (purple)
            Self::PoseFactor => "#c6aof6",     // maroon (red)
        }
    }

    pub fn shape(&self) -> &'static str {
        match self {
            Self::Variable { .. } => "circle",
            _ => "square",
        }
    }

    pub fn width(&self) -> f64 {
        match self {
            Self::Variable { .. } => 0.8,
            _ => 0.2,
        }
    }
}

pub struct Edge {
    pub from: usize,
    pub to:   usize,
}

pub trait Graph {
    fn export_data(&self) -> (Vec<Node>, Vec<Edge>);
}
