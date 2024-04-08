use bevy::prelude::*;
// use bevy_more_shapes::Cylinder;
// use bevy_math::RegularPolygon;
use serde::{Deserialize, Serialize};

use crate::{
    asset_loader::SceneAssets,
    config::{environment::PlaceableShape, Environment, Obstacle, Obstacles},
};

pub struct GenMapPlugin;

impl Plugin for GenMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (build_tile_grid, build_obstacles));
    }
}
#[derive(Debug, Serialize, Deserialize, Component)]
#[serde(rename_all = "kebab-case")]
pub struct TileCoordinates {
    row: usize,
    col: usize,
}

impl TileCoordinates {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// **Bevy** [`Startup`] _system_.
/// Takes the [`Environment`] configuration and generates all specified
/// ['Obstacles'].
///
/// [`Obstacles`] example:
/// ```rust
/// Obstacles(vec![
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::circle(0.1, (0.5, 0.5)),
///         Angle::new(0.0).unwrap(),
///     ),
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::triangle(0.1, 0.1, 0.5),
///         Angle::new(0.0).unwrap(),
///     ),
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::square(0.1, (0.75, 0.5)),
///         Angle::new(0.0).unwrap(),
///     ),
/// ]),
/// ```
///
/// Placement of all shapes is given as a `(x, y)` percentage local to a
/// specific tile
fn build_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Res<Environment>,
    scene_assets: Res<SceneAssets>,
) {
    let tile_grid = &env_config.tiles.grid;
    let tile_size = env_config.tile_size();
    let obstacle_height = env_config.obstacle_height();

    let grid_offset_x = tile_grid.cols() as f32 / 2.0 - 0.5;
    let grid_offset_z = tile_grid.rows() as f32 / 2.0 - 0.5;

    info!("Spawning obstacles");
    info!("{:?}", env_config.obstacles);
    info!(
        "env_config.obstacles.iter().count() = {:?}",
        env_config.obstacles.iter().count()
    );

    let obstacles_to_spawn = env_config.obstacles.iter().map(|obstacle| {
        let TileCoordinates { row, col } = obstacle.tile_coordinates;

        info!("Spawning obstacle at {:?}", (row, col));

        let tile_offset_x = col as f32;
        let tile_offset_z = row as f32;

        let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
        let offset_z = (tile_offset_z - grid_offset_z) * tile_size;

        let pos_offset = tile_size / 2.0;

        // Construct the correct shape
        match obstacle.shape {
            PlaceableShape::Circle { radius, center } => {
                let center = Vec3::new(
                    offset_x + center.x.get() as f32 * tile_size - pos_offset,
                    obstacle_height / 2.0,
                    offset_z + center.y.get() as f32 * tile_size - pos_offset,
                );

                info!("Spawning circle: r = {}, at {:?}", radius, center);
                let radius = radius.get() as f32 * tile_size;

                let mesh = meshes.add(Cylinder::new(radius, obstacle_height));
                let transform = Transform::from_translation(center);

                info!(
                    "Spawning cylinder: r = {}, h = {}, at {:?}",
                    radius, obstacle_height, transform
                );

                Some((mesh, transform))
            }
            PlaceableShape::RegularPolygon {
                sides,
                side_length,
                translation,
            } => {
                if sides != 4 {
                    unimplemented!("Only squares are currently supported")
                }
                let center = Vec3::new(
                    offset_x + translation.x.get() as f32 * tile_size - pos_offset,
                    obstacle_height / 2.0,
                    offset_z + translation.y.get() as f32 * tile_size - pos_offset,
                );

                info!(
                    "Spawning regular polygon: sides = {}, side_length = {}, at {:?}",
                    sides, side_length, center
                );

                // let mesh = meshes.add(Cuboid::new(
                //     side_length.get() as f32 * tile_size,
                //     obstacle_height,
                //     side_length.get() as f32 * tile_size,
                // ));
                let mesh = meshes.add(Mesh::from(bevy_more_shapes::Cylinder {
                    height: obstacle_height,
                    radius_bottom: side_length.get() as f32 * tile_size / 2.0,
                    radius_top: side_length.get() as f32 * tile_size / 2.0,
                    radial_segments: sides as u32,
                    height_segments: 1,
                }));
                let transform = Transform::from_translation(center);

                Some((mesh, transform))
            }
            _ => None,
        }
    });

    obstacles_to_spawn
        .filter_map(|obstacle| obstacle) // filter out None
        .for_each(|(mesh, transform)| {
            commands.spawn(PbrBundle {
                mesh,
                material: scene_assets.materials.obstacle.clone(),
                transform,
                ..Default::default()
            });
        });

    // exit
    // std::process::exit(0);
}

/// **Bevy** [`Startup`] _system_.
/// Takes the [`Environment`] configuration and generates a map.
///
/// Transforms an input like:
/// ```text
/// ┌┬┐
/// ┘└┼┬
///   └┘
/// ```
/// Into visual meshes, defining the physical boundaries of the map
/// - The lines are not walls, but paths
/// - Where the empty space are walls/obstacles
///
/// Each tile e.g. tile (0,0) in the above grid "┌" or (3,1) "┬"
/// - Transforms into a 1x1 section of the map - later to be scaled
/// - Each tile's world position is calculated from the tile's position in the
///   grid
///     - Such that the map is centered
/// - Uses the `Environment.width` to determine the width of the paths,
///    - Otherwise, the empty space is filled with solid meshes
fn build_tile_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Res<Environment>,
    scene_assets: Res<SceneAssets>,
) {
    let tile_grid = &env_config.tiles.grid;

    let obstacle_height = env_config.obstacle_height();
    let obstacle_y = obstacle_height / 2.0;

    let tile_size = env_config.tile_size();

    let path_width = env_config.path_width();
    let base_dim = tile_size * (1.0 - path_width) / 2.0;

    // offset caused by the size of the grid
    // - this centers the map
    let grid_offset_x = tile_grid.cols() as f32 / 2.0 - 0.5;
    let grid_offset_z = tile_grid.rows() as f32 / 2.0 - 0.5;

    let pos_offset = (base_dim + path_width * tile_size) / 2.0;

    for (y, row) in tile_grid.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            // offset of the individual tile in the grid
            // used in all match cases
            let tile_offset_x = x as f32;
            let tile_offset_z = y as f32;

            // total offset caused by grid and tile
            let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
            let offset_z = (tile_offset_z - grid_offset_z) * tile_size;
            if let Some(mesh_transforms) = match tile {
                '─' => {
                    // Horizontal straight path
                    // - 2 equal-sized larger cuboid on either side, spanning the entire width of
                    //   the tile

                    Some(vec![
                        (
                            // left side
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '│' => {
                    // Vertical straight path
                    // - 2 equal-sized larger cuboid on either side, spanning the entire height of
                    //   the tile

                    Some(vec![
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╴' => {
                    // Termination from the left
                    // - 2 larger cuboids on the top and bottom, spanning the entire width of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the right, to terminate the path

                    Some(vec![
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right plug
                            meshes.add(Cuboid::new(
                                base_dim,
                                obstacle_height,
                                path_width * tile_size,
                            )),
                            // right plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╶' => {
                    // Termination from the right
                    // - 2 larger cuboids on the top and bottom, spanning the entire width of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the left, to terminate the path

                    Some(vec![
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left plug
                            meshes.add(Cuboid::new(
                                base_dim,
                                obstacle_height,
                                path_width * tile_size,
                            )),
                            // left plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╷' => {
                    // Termination from the bottom
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the top, to terminate the path

                    Some(vec![
                        (
                            // left
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top plug
                            meshes.add(Cuboid::new(
                                path_width * tile_size,
                                obstacle_height,
                                base_dim,
                            )),
                            // top plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '╵' => {
                    // Termination from the top
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the bottom, to terminate the path

                    Some(vec![
                        (
                            // left
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom plug
                            meshes.add(Cuboid::new(
                                path_width * tile_size,
                                obstacle_height,
                                base_dim,
                            )),
                            // bottom plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┌' => {
                    // Top right hand turn
                    // - 1 cube in the bottom right corner
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the top side, spanning from the right to the above
                    //   cuboid

                    Some(vec![
                        (
                            // bottom right cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '┐' => {
                    // Top left hand turn
                    // - 1 cube in the bottom left corner
                    // - 1 larger cuboid on the right hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the top side, spanning from the left to the above cuboid

                    Some(vec![
                        (
                            // bottom left cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '└' => {
                    // Bottom right hand turn
                    // - 1 cube in the top right corner
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the bottom side, spanning from the right to the above
                    //   cuboid

                    Some(vec![
                        (
                            // top right cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┘' => {
                    // Bottom left hand turn
                    // - 1 cube in the top left corner
                    // - 1 larger cuboid on the right hand side, spannding the entire height of the
                    //   tile
                    // - 1 larger cuboid on the bottom side, spanning from the left to the above
                    //   cuboid

                    Some(vec![
                        (
                            // top left cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┬' => {
                    // Top T-junction
                    // - 2 equal-sized cubes, one in each bottom corner
                    // - 1 larger cuboid in the top center, spanning the entire width of the tile

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let top = meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // bottom left cube
                            cube.clone(),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube.clone(),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // top center cuboid
                            top,
                            // top center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '┴' => {
                    // Bottom T-junction
                    // - 2 equal-sized cubes, one in each top corner
                    // - 1 larger cuboid in the bottom center, spanning the entire width of the tile

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let bottom = meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // top left cube
                            cube.clone(),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right cube
                            cube.clone(),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom center cuboid
                            bottom,
                            // bottom center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '├' => {
                    // Right T-junction
                    // - 2 equal-sized cubes, one in each right corner
                    // - 1 larger cuboid in the left center, spanning the entire height of the tile

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let left = meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size));

                    Some(vec![
                        (
                            // top right cube
                            cube.clone(),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube.clone(),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left center cuboid
                            left,
                            // left center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '┤' => {
                    // Left T-junction
                    // - 2 equal-sized cubes, one in each left corner
                    // - 1 larger cuboid in the right center, spanning the entire height of the tile

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let right = meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size));

                    Some(vec![
                        (
                            // top left cube
                            cube.clone(),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left cube
                            cube.clone(),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right center cuboid
                            right,
                            // right center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '┼' => {
                    // 4-way intersection
                    // - 4 equal-sized cubes, one in each corner
                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // top left
                            cube.clone(),
                            // top left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right
                            cube.clone(),
                            // top right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left
                            cube.clone(),
                            // bottom left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right
                            cube.clone(),
                            // bottom right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                ' ' => {
                    // Empty space
                    // - 1 larger cuboid, spanning the entire tile
                    Some(vec![(
                        meshes.add(Cuboid::new(tile_size, obstacle_height, tile_size)),
                        Transform::from_translation(Vec3::new(offset_x, obstacle_y, offset_z)),
                    )])
                }
                _ => None,
            } {
                mesh_transforms.iter().for_each(|(mesh, transform)| {
                    commands.spawn((
                        PbrBundle {
                            mesh: mesh.clone(),
                            transform: *transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        TileCoordinates::new(x, y),
                    ));
                });
            }
        }
    }
}
