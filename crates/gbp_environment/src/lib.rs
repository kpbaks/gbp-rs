use std::path::Path;

use angle::Angle;
use bevy::ecs::{component::Component, system::Resource};
use derive_more::IntoIterator;
use gbp_geometry::RelativePoint;
use gbp_linalg::Float;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[serde(rename_all = "kebab-case")]
pub struct TileCoordinates {
    pub row: usize,
    pub col: usize,
}

impl TileCoordinates {
    #[must_use]
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoIterator)]
#[into_iterator(owned, ref)]
#[serde(rename_all = "kebab-case")]
pub struct TileGrid(Vec<String>);

impl TileGrid {
    pub fn new(tiles: Vec<impl Into<String>>) -> Self {
        Self(tiles.into_iter().map(Into::into).collect())
    }

    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns number of rows in the tilegrid
    #[inline]
    pub fn nrows(&self) -> usize {
        self.0.len()
    }

    /// Returns number of columns in the tilegrid
    #[inline]
    pub fn ncols(&self) -> usize {
        self.0[0].chars().count()
    }

    /// Returns the tile at the given coordinates
    pub fn get_tile(&self, row: usize, col: usize) -> Option<char> {
        self.0.get(row).and_then(|r| r.chars().nth(col))
    }

    // /// override the index operator to allow for easy access to the grid
    // pub fn get(&self, row: usize, col: usize) -> Option<char> {
    //     self.0.get(row).and_then(|r| r.chars().nth(col))
    // }
}

// impl std::ops::Index<(usize, usize)> for TileGrid {
//     type Output = char;

//     fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
//         &self
//             .0
//             .get(row)
//             .and_then(|r| r.chars().nth(col))
//             .expect("index is within grid")
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Rotation(Angle);

impl Rotation {
    /// Create a new `Rotation` from a given degree
    ///
    /// # Panics
    ///
    /// If `degree` is not in [0.0, 360.0]
    #[must_use]
    pub fn new(degree: Float) -> Self {
        Self(Angle::from_degrees(degree).expect("Invalid angle"))
    }
}

impl Rotation {
    /// Get the rotation in radians
    #[inline]
    pub const fn as_radians(&self) -> Float {
        self.0.as_radians()
    }

    /// Get the rotation in degrees
    #[inline]
    pub fn as_degrees(&self) -> Float {
        self.0.as_degrees()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        mid_point:   Float,
        /// Where to place the center of the triangle
        translation: RelativePoint,
    },
    RegularPolygon {
        /// The number of sides of the polygon
        sides:       usize,
        /// Side length of the polygon
        side_length: StrictlyPositiveFinite<Float>,
        /// Where to place the center of the polygon
        translation: RelativePoint,
    },
    Rectangle {
        /// The width of the rectangle
        /// This is a value in the range [0, 1]
        width:       StrictlyPositiveFinite<Float>,
        /// The height of the rectangle
        /// This is a value in the range [0, 1]
        height:      StrictlyPositiveFinite<Float>,
        /// The center of the rectangle
        translation: RelativePoint,
    },
}

impl PlaceableShape {
    /// Create a new `Self::Circle`
    ///
    /// # Panics
    ///
    /// If `center` is not a relative point i.e. within interval ([0.0, 1.0],
    /// [0.0, 1.0])
    #[allow(clippy::unwrap_used)]
    pub fn circle(radius: StrictlyPositiveFinite<Float>, center: (Float, Float)) -> Self {
        Self::Circle {
            // radius: StrictlyPositiveFinite::<Float>::new(radius).unwrap(),
            radius,
            center: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    /// Create a new `Self::Triangle`
    ///
    /// # Panics
    ///
    /// If `translation` is not a relative point i.e. within interval ([0.0,
    /// 1.0], [0.0, 1.0])
    #[allow(clippy::unwrap_used)]
    pub fn triangle(
        // base_length: Float,
        base_length: StrictlyPositiveFinite<Float>,
        // height: Float,
        height: StrictlyPositiveFinite<Float>,
        mid_point: Float,
        translation: (Float, Float),
    ) -> Self {
        Self::Triangle {
            // base_length: StrictlyPositiveFinite::<Float>::new(base_length).unwrap(),
            base_length,
            // height: StrictlyPositiveFinite::<Float>::new(height).unwrap(),
            height,
            mid_point,
            translation: RelativePoint::new(translation.0, translation.1).unwrap(),
        }
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn rectangle(width: Float, height: Float, center: (Float, Float)) -> Self {
        Self::Rectangle {
            width:       StrictlyPositiveFinite::<Float>::new(width).unwrap(),
            height:      StrictlyPositiveFinite::<Float>::new(height).unwrap(),
            translation: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn square(side_length: Float, center: (Float, Float)) -> Self {
        Self::RegularPolygon {
            sides:       4,
            side_length: StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            translation: RelativePoint::new(center.0, center.1).unwrap(),
        }
    }

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn regular_polygon(sides: usize, side_length: Float, translation: (Float, Float)) -> Self {
        Self::RegularPolygon {
            sides,
            side_length: StrictlyPositiveFinite::<Float>::new(side_length).unwrap(),
            translation: RelativePoint::new(translation.0, translation.1).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Obstacle {
    /// The shape to be placed as an obstacle
    pub shape: PlaceableShape,
    /// Rotation of the obstacle in degrees around the up-axis
    pub rotation: Rotation,
    /// Which tile in the grid the obstacle should be placed
    pub tile_coordinates: TileCoordinates,
}

impl Obstacle {
    /// Create a new `Obstacle`
    ///
    /// # Panics
    ///
    /// If `rotation` is not a normalized angle, i.e. within [0.0, 2pi]
    #[must_use]
    pub fn new((row, col): (usize, usize), shape: PlaceableShape, rotation: Float) -> Self {
        Self {
            tile_coordinates: TileCoordinates::new(row, col),
            shape,
            rotation: Rotation(Angle::new(rotation).expect("Invalid angle")),
        }
    }
}

/// Struct to represent a list of shapes that can be placed in the map [`Grid`]
/// Each entry contains a [`Cell`] and a [`PlaceableShape`]
/// - The [`Cell`] represents which tile in the [`Grid`] the shape should be
///   placed
/// - The [`PlaceableShape`] represents the shape to be placed, and the local
///   cell translation
#[derive(Debug, Clone, Serialize, Deserialize, IntoIterator)]
#[serde(rename_all = "kebab-case")]
#[into_iterator(owned, ref)]
pub struct Obstacles(Vec<Obstacle>);

impl Obstacles {
    /// Create a new empty vector of [`Obstacle`]
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn iter(&self) -> std::slice::Iter<Obstacle> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TileSettings {
    pub tile_size:       f32,
    pub path_width:      f32,
    pub obstacle_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Tiles {
    pub grid:     TileGrid,
    pub settings: TileSettings,
}

impl Tiles {
    /// Create an empty `Tiles`
    #[must_use]
    pub fn empty() -> Self {
        Self {
            grid:     TileGrid::new(vec!["█"]),
            settings: TileSettings {
                tile_size:       0.0,
                path_width:      0.0,
                obstacle_height: 0.0,
            },
        }
    }

    /// Set the tile size
    #[must_use]
    pub const fn with_tile_size(mut self, tile_size: f32) -> Self {
        self.settings.tile_size = tile_size;
        self
    }

    /// Set the obstacle height
    #[must_use]
    pub const fn with_obstacle_height(mut self, obstacle_height: f32) -> Self {
        self.settings.obstacle_height = obstacle_height;
        self
    }
}

#[derive(Debug, clap::ValueEnum, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnvironmentType {
    #[default]
    Intersection,
    Intermediate,
    Complex,
    Circle,
    Maze,
    Test,
}

/// **Bevy** [`Resource`]
/// The environment configuration for the simulation
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Environment {
    pub tiles:     Tiles,
    pub obstacles: Obstacles,
}

impl Default for Environment {
    fn default() -> Self {
        Self::intersection()
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
    #[error("Environment matrix representation is empty")]
    EmptyGrid,
    #[error("Environment matrix representation has rows of different lengths")]
    DifferentLengthRows,
}

impl Environment {
    /// Attempt to parse an [`Environment`] from a YAML file at `path`
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|contents| Self::parse(contents.as_str()))
    }

    /// Attempt to parse an [`Environment`] from a YAML encoded string
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        // ron::from_str::<Environment>(contents)
        //     .map_err(|span| span.code)?
        //     .validate()
        //     .map_err(Into::into)
        // with yaml

        serde_yaml::from_str::<Self>(contents)
            .map_err(Into::into)
            .and_then(|env| env.validate().map_err(Into::into))
    }

    /// Ensure that the [`Environment`] is valid
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. The matrix representation is not empty
    /// 2. All rows in the matrix representation are the same length
    pub fn validate(self) -> Result<Self, EnvironmentError> {
        if self.tiles.grid.is_empty() {
            Err(EnvironmentError::EmptyGrid)
        } else if self
            .tiles
            .grid
            .iter()
            .any(|row| row.chars().count() != self.tiles.grid.ncols())
        {
            Err(EnvironmentError::DifferentLengthRows)
        } else {
            Ok(self)
        }
    }

    #[must_use]
    pub fn new(
        matrix_representation: Vec<String>,
        path_width: f32,
        obstacle_height: f32,
        tile_size: f32,
    ) -> Self {
        Self {
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

    #[must_use]
    pub fn intersection() -> Self {
        Self {
            tiles:     Tiles {
                grid:     TileGrid::new(vec!["┼"]),
                settings: TileSettings {
                    tile_size:       100.0,
                    path_width:      0.1325,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn intermediate() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌┬┐ ",
                    "┘└┼┬",
                    "  └┘",
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

    #[must_use]
    #[rustfmt::skip]
    pub fn complex() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌─┼─┬─┐┌",
                    "┼─┘┌┼┬┼┘",
                    "┴┬─┴┼┘│ ",
                    "┌┴┐┌┼─┴┬",
                    "├─┴┘└──┘",
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

    #[must_use]
    #[rustfmt::skip]
    pub fn maze() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "               ",
                    " ╶─┬─┐┌─────┬┐ ",
                    " ┌─┤┌┤│╷╶──┬┘│ ",
                    " │╷│╵├┤├─┬┬┴┬┤ ",
                    " └┤├─┘││╷╵├─┘│ ",
                    " ╷│╵╷╶┤│├┐└╴┌┘ ",
                    " │├─┴╴│╵│└──┤╷ ",
                    " └┤┌─┐└┬┘┌─┐└┘ ",
                    " ┌┴┤╷├╴│┌┤╷└─┐ ",
                    " │┌┤├┘┌┘││└──┤ ",
                    " ╵│╵├┬┘┌┘└──┐╵ ",
                    " ┌┘╶┘├─┴─┐╷╷└┐ ",
                    " └─┬─┴──┐├┘├─┘ ",
                    " ┌┐│╷┌─╴││╶┘╶┐ ",
                    " │└┼┘├──┘├──┬┤ ",
                    " ╵╶┴─┘╶──┴──┴┘ ",
                    "               ",
                ]),
                settings: TileSettings {
                    tile_size: 10.0,
                    path_width: 0.75,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn test() -> Self {
        Self {
            tiles: Tiles {
                grid: TileGrid::new(vec![
                    "┌┬┐├",
                    "└┴┘┤",
                    "│─ ┼",
                    "╴╵╶╷",
                ]),
                settings: TileSettings {
                    tile_size: 50.0,
                    path_width: 0.1325,
                    obstacle_height: 1.0,
                },
            },
            obstacles: Obstacles::empty(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn circle() -> Self {
        Self {
            tiles:     Tiles::empty()
                .with_tile_size(100.0)
                .with_obstacle_height(1.0),
            obstacles: Obstacles(vec![
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.0525, (0.625, 0.60125)),
                    0.0,
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.035, (0.44125, 0.57125)),
                    0.0,
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::regular_polygon(4, 0.0225, (0.4835, 0.428)),
                    0.0,
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::rectangle(0.0875, 0.035, (0.589, 0.3965)),
                    0.0,
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::triangle(
                        0.03.try_into().expect("positive and finite"),
                        0.0415.try_into().expect("positive and finite"),
                        0.575,
                        (0.5575, 0.5145),
                    ),
                    0.0,
                ),
                Obstacle::new(
                    (0, 0),
                    PlaceableShape::triangle(
                        0.012.try_into().expect("positive and finite"),
                        0.025.try_into().expect("positive and finite"),
                        1.25,
                        (0.38, 0.432),
                    ),
                    5.225,
                ),
            ]),
        }
    }

    pub const fn path_width(&self) -> f32 {
        self.tiles.settings.path_width
    }

    pub const fn obstacle_height(&self) -> f32 {
        self.tiles.settings.obstacle_height
    }

    pub const fn tile_size(&self) -> f32 {
        self.tiles.settings.tile_size
    }
}