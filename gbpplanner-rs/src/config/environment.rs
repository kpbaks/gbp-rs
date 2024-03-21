use angle::Angle;
use bevy::ecs::{system::Resource, world::error};
use gbp_linalg::Float;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

use super::geometry::RelativePoint;
use crate::environment::TileCoordinates;

// use super::geometry::Shape;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TileGrid(Vec<String>);

impl TileGrid {
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn rows(&self) -> usize {
        self.0.len()
    }

    pub fn cols(&self) -> usize {
        self.0[0].chars().count()
    }

    // override the index operator to allow for easy access to the grid
    pub fn get(&self, row: usize, col: usize) -> Option<char> {
        self.0.get(row).and_then(|r| r.chars().nth(col))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Rotation(Angle);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaceableShape {
    Circle {
        /// The radius of the circle
        /// This is a value in the range [0, 1]
        radius: StrictlyPositiveFinite<Float>,
        /// The center of the circle,
        center: RelativePoint,
    },
    Triangle {
        /// The length of the base of the triangle
        /// This is a value in the range [0, 1]
        base_length: StrictlyPositiveFinite<Float>,
        /// The height of the triangle
        /// Intersects the base perpendicularly at the mid-point
        height:      StrictlyPositiveFinite<Float>,
        /// The mid-point of the base of the triangle
        /// This is a value in the range [0, 1]
        /// Defines where the height of the triangle intersects the base
        /// perpendicularly
        mid_point:   UnitInterval,
    },
    RegularPolygon {
        /// The number of sides of the polygon
        sides:       usize,
        /// Side length of the polygon
        side_length: StrictlyPositiveFinite<Float>,
        /// Where to place the center of the polygon
        translation: RelativePoint,
    },
}

impl PlaceableShape {
    pub fn circle(radius: Float, center: (Float, Float)) -> Self {
        PlaceableShape::Circle {
            radius: StrictlyPositiveFinite::<Float>::new(radius).unwrap(),
            center: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    pub fn triangle(base_length: Float, height: Float, mid_point: Float) -> Self {
        PlaceableShape::Triangle {
            base_length: StrictlyPositiveFinite::<Float>::new(base_length).unwrap(),
            height:      StrictlyPositiveFinite::<Float>::new(height).unwrap(),
            mid_point:   UnitInterval::new(mid_point).unwrap(),
        }
    }

    pub fn rectangle(width: Float, height: Float, center: (Float, Float)) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let top = RelativePoint::new(center.0, center.1 + half_height).unwrap();
        let left = RelativePoint::new(center.0 - half_width, center.1 - half_height).unwrap();
        let right = RelativePoint::new(center.0 + half_width, center.1 - half_height).unwrap();
        PlaceableShape::RegularPolygon {
            sides:       4,
            side_length: StrictlyPositiveFinite::<Float>::new(width).unwrap(),
            translation: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    pub fn square(side_length: Float, center: (Float, Float)) -> Self {
        let half_side = side_length / 2.0;
        let top = RelativePoint::new(center.0, center.1 + half_side).unwrap();
        let left = RelativePoint::new(center.0 - half_side, center.1 - half_side).unwrap();
        let right = RelativePoint::new(center.0 + half_side, center.1 - half_side).unwrap();
        PlaceableShape::RegularPolygon {
            sides:       4,
            side_length: StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            translation: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    pub fn regular_polygon(sides: usize, side_length: Float, translation: (Float, Float)) -> Self {
        PlaceableShape::RegularPolygon {
            sides,
            side_length: StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            translation: RelativePoint::new(translation.0, translation.1).unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Obstacle {
    pub tile_coordinates: TileCoordinates,
    pub shape: PlaceableShape,
    pub rotation: Rotation,
}

impl Obstacle {
    pub fn new((row, col): (usize, usize), shape: PlaceableShape, rotation: Angle) -> Self {
        Obstacle {
            tile_coordinates: TileCoordinates::new(row, col),
            shape,
            rotation: Rotation(rotation),
        }
    }
}

/// Struct to represent a list of shapes that can be placed in the map [`Grid`]
/// Each entry contains a [`Cell`] and a [`PlaceableShape`]
/// - The [`Cell`] represents which tile in the [`Grid`] the shape should be
///   placed
/// - The [`PlaceableShape`] represents the shape to be placed, and the local
///   cell translation
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Obstacles(Vec<Obstacle>);

impl Obstacles {
    pub fn empty() -> Self {
        Obstacles(Vec::new())
    }

    pub fn iter(&self) -> std::slice::Iter<Obstacle> {
        self.0.iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TileSettings {
    pub tile_size:       f32,
    pub path_width:      f32,
    pub obstacle_height: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Tiles {
    pub grid:     TileGrid,
    pub settings: TileSettings,
}

impl Tiles {
    pub fn empty() -> Self {
        Tiles {
            grid:     TileGrid(vec!["█".to_string()]),
            settings: TileSettings {
                tile_size:       0.0,
                path_width:      0.0,
                obstacle_height: 0.0,
            },
        }
    }

    pub fn with_tile_size(mut self, tile_size: f32) -> Self {
        self.settings.tile_size = tile_size;
        self
    }

    pub fn with_obstacle_height(mut self, obstacle_height: f32) -> Self {
        self.settings.obstacle_height = obstacle_height;
        self
    }
}

#[derive(clap::ValueEnum, Default, Clone, Copy)]
pub enum EnvironmentType {
    #[default]
    Intersection,
    Intermediate,
    Complex,
    Circle,
}

/// **Bevy** [`Resource`]
/// The environment configuration for the simulation
#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Environment {
    pub tiles:     Tiles,
    pub obstacles: Obstacles,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::intersection()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    InvalidEnvironment(#[from] EnvironmentError),
}

#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    EmptyGrid,
    DifferentLengthRows,
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentError::EmptyGrid => {
                write!(f, "Environment matrix representation is empty")
            }
            EnvironmentError::DifferentLengthRows => {
                write!(
                    f,
                    "Environment matrix representation has rows of different lengths"
                )
            }
        }
    }
}

impl Environment {
    /// Attempt to parse an [`Environment`] from a RON file at `path`
    /// Returns `Err(ParseError)` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn from_file(path: &std::path::Path) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|contents| Self::parse(contents.as_str()))
    }

    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        // ron::from_str::<Environment>(contents)
        //     .map_err(|span| span.code)?
        //     .validate()
        //     .map_err(Into::into)
        // with yaml

        serde_yaml::from_str::<Environment>(contents)
            .map_err(Into::into)
            .and_then(|env| env.validate().map_err(Into::into))
    }

    /// Ensure that the [`Environment`] is valid
    /// 1. The matrix representation is not empty
    /// 2. All rows in the matrix representation are the same length
    pub fn validate(self) -> Result<Self, EnvironmentError> {
        if self.tiles.grid.is_empty() {
            Err(EnvironmentError::EmptyGrid)
        } else if self
            .tiles
            .grid
            .iter()
            .any(|row| row.chars().count() != self.tiles.grid.cols())
        {
            Err(EnvironmentError::DifferentLengthRows)
        } else {
            Ok(self)
        }
    }

    pub fn new(
        matrix_representation: Vec<String>,
        path_width: f32,
        obstacle_height: f32,
        tile_size: f32,
    ) -> Self {
        Environment {
            tiles:     Tiles {
                grid:     TileGrid(matrix_representation),
                settings: TileSettings {
                    tile_size,
                    path_width,
                    obstacle_height,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    pub fn intersection() -> Self {
        Environment {
            tiles:     Tiles {
                grid:     TileGrid(vec!["┼".to_string()]),
                settings: TileSettings {
                    tile_size:       100.0,
                    path_width:      0.1325,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[rustfmt::skip]
    pub fn intermediate() -> Self {
        Environment {
            tiles: Tiles {
                grid: TileGrid(vec![
                    "┌┬┐ ".to_string(),
                    "┘└┼┬".to_string(),
                    "  └┘".to_string()
                ]),
                settings: TileSettings {
                    tile_size: 50.0,
                    path_width: 0.1325,
                    obstacle_height: 1.0,
                }
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[rustfmt::skip]
    pub fn complex() -> Self {
        Environment {
            tiles: Tiles {
                grid: TileGrid(vec![
                    "┌─┼─┬─┐┌".to_string(),
                    "┼─┘┌┼┬┼┘".to_string(),
                    "┴┬─┴┼┘│ ".to_string(),
                    "┌┴┐┌┼─┴┬".to_string(),
                    "├─┴┘└──┘".to_string(),
                ]),
                settings: TileSettings {
                    tile_size: 25.0,
                    path_width: 0.4,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    pub fn circle() -> Self {
        Environment {
            tiles:     Tiles::empty()
                .with_tile_size(100.0)
                .with_obstacle_height(1.0),
            obstacles: Obstacles(vec![
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::circle(0.1, (0.5, 0.5)),
                    Angle::new(0.0).unwrap(),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::triangle(0.1, 0.1, 0.5),
                    Angle::new(0.0).unwrap(),
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::square(0.1, (0.75, 0.5)),
                    Angle::new(0.0).unwrap(),
                ),
            ]),
        }
    }

    pub fn path_width(&self) -> f32 {
        self.tiles.settings.path_width
    }

    pub fn obstacle_height(&self) -> f32 {
        self.tiles.settings.obstacle_height
    }

    pub fn tile_size(&self) -> f32 {
        self.tiles.settings.tile_size
    }
}