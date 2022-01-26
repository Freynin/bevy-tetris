use bevy::{
    prelude::*,
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UpdateTimer(Timer::from_seconds(0.08, true)))
        .insert_resource(SoftDropTimer(Timer::from_seconds(0.450, true)))
        .insert_resource(PrintInfoTimer(Timer::from_seconds(0.8, true)))

        .add_startup_system(setup)
        //.add_system(print_info)
        .add_system(move_current_tetromino)
        .add_system(update_block_sprites)
        .add_system(clear_full_layers)
        .run();
}

struct SoftDropTimer(Timer);

struct PrintInfoTimer(Timer);

struct UpdateTimer(Timer);
// Base entity, everything is made out of blocks
#[derive(Component)]
struct Block {
    color: Color,
}
#[derive(Component)]
struct Matrix {
    width: i32,
    height: i32,
}

// Holds a block's position within a tetromino for rotation
#[derive(Component)]
#[derive(Debug)]
struct MatrixPosition {
    x: i32,
    y: i32,
}

// A block can be part of a tetromino. Stores the block's index within that
// tetromino for the purpose of rotation.
#[derive(Component)]
#[derive(Debug)]
struct Tetromino {
    tetromino_type: TetrominoType,
    index: MatrixPosition,
}

// A block can be part of the currently controlled tetromino.
#[derive(Component)]
struct CurrentTetromino;



// A block can be part of the heap.
#[derive(Component)]
struct Heap;

impl Block {
    const SIZE: f32 = 25.0;
}

#[derive(Component)]
struct ScoreText;



fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>
) {
    let matrix = Matrix {
        width: 10,
        height: 22,
    };

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn().insert_bundle(UiCameraBundle::default())
    ;

    spawn_current_tetromino(&mut commands, &matrix, &mut materials);
    let mysprite = Sprite {
        color: Color::rgb(0.0, 0.0, 0.0).into(),
        flip_x: false,
        flip_y: false,
        custom_size: Some(Vec2::new(matrix.width as f32 * Block::SIZE, matrix.height as f32 * Block::SIZE)),
    };
    commands
        .spawn().insert_bundle(SpriteBundle {
            sprite: mysprite,
            ..Default::default()
        })
        .insert(matrix)
    ;
    commands.spawn_bundle(TextBundle {
                text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });
}

fn _print_info(
    time: Res<Time>,
    mut timer: ResMut<PrintInfoTimer>,
    mut _matrix_query: Query<(&Matrix, &Sprite, &Transform)>,
    mut current_query: Query<(Entity, &MatrixPosition, &Tetromino, &CurrentTetromino)>
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        for (entity, position, tetromino, _current) in current_query.iter_mut() {
            println!("Current matrix_pos: {:?}", position);
            println!("Current tetromino: {:?}", tetromino);
            println!("{:?}", entity);
        }
    }
}

fn move_current_tetromino(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut soft_drop_timer: ResMut<SoftDropTimer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut matrix_query: Query<&Matrix>,
    mut current_query: Query<(Entity, &mut MatrixPosition, &mut Tetromino, &CurrentTetromino), Without<Heap>>,
    mut heap_query: Query<(Entity, &mut MatrixPosition, &Heap)>
) {
    soft_drop_timer.0.tick(time.delta()).just_finished();
    
    // Store current positions in map by entity ID
    let mut prev_positions: HashMap<u32, (i32, i32)> = HashMap::new();
    for (entity, position, _tetromino, _current) in current_query.iter_mut() {
        prev_positions.insert(entity.id(), (position.x, position.y));
    }
    // Press up to drop all the way
    if keyboard_input.just_pressed(KeyCode::I) || keyboard_input.just_pressed(KeyCode::Up) {
        while check_tetromino_positions(&mut current_query, &mut heap_query) {
            for (_entity, mut position, _tetromino, _current) in current_query.iter_mut() {
                position.y -= 1;
            }
        }
        // We intersect, so we move up and move tetromino blocks into heap
        for (entity, mut position, _tetromino, _current) in current_query.iter_mut() {
            position.y += 1;
            commands.entity(entity)
                .remove::<CurrentTetromino>()
                .insert(Heap)
                ;
            //commands.remove_one::<CurrentTetromino>(entity);
            //commands.insert_one(entity, Heap);
        }
        //clear_full_layers(commands, &mut heap_query);
        for matrix in matrix_query.iter_mut() {
            spawn_current_tetromino(&mut commands, matrix, &mut materials);
        }

        return;
    }

    // Movement

    let mut move_x = 0;
    let mut move_y = 0;
    if keyboard_input.just_pressed(KeyCode::J) || keyboard_input.just_pressed(KeyCode::Left) {
        move_x -= 1;
    }

    if keyboard_input.just_pressed(KeyCode::L) || keyboard_input.just_pressed(KeyCode::Right) {
        move_x += 1;
    }

    if keyboard_input.just_pressed(KeyCode::K) || keyboard_input.just_pressed(KeyCode::Down) || soft_drop_timer.0.just_finished() {
        move_y -= 1;
    }

    let mut should_rotate: Option<bool> = None;
    if keyboard_input.just_pressed(KeyCode::X) {
        should_rotate = Some(true);
    }

    if keyboard_input.just_pressed(KeyCode::Z) {
        should_rotate = Some(false);
    }

    let mut x_over = 0;
    let y_over = 0;

    for (_entity, mut position, mut tetromino, _current) in current_query.iter_mut() {
        let mut move_x = move_x;
        let mut move_y = move_y;

        // Rotation
        if let Some(clockwise) = should_rotate {
            let prev_index_x = tetromino.index.x;
            let prev_index_y = tetromino.index.y;

            let matrix_size = Tetromino::SIZES[tetromino.tetromino_type as usize];
            rotate_tetromino_block(&mut tetromino, matrix_size, clockwise);

            move_x += tetromino.index.x - prev_index_x;
            move_y += tetromino.index.y - prev_index_y;
        }

        // Bounds
        for matrix in matrix_query.iter_mut() {
            if position.x + move_x < 0 {
                x_over = (position.x + move_x).min(x_over);

            } else if position.x + move_x >= matrix.width {
                x_over = ((position.x + move_x) - matrix.width + 1).max(x_over);
            }
        }

        position.x += move_x;
        position.y += move_y;
    }

    for (_entity, mut position, mut _tetromino, _current) in current_query.iter_mut() {
        position.x -= x_over;
        position.y -= y_over;
    }

    // TODO: Probably better off setting the matrix up so you can index into it to look for occupied spots around the current tetromino
    // Check if any blocks in tetromino are overlapping with heap
    if !check_tetromino_positions(&mut current_query, &mut heap_query) {
        let mut should_revert = true;

        if let Some(_) = should_rotate {
            let try_moves = [
                ( 1,  0),
                ( 2,  0),
                (-1,  0),
                (-2,  0),
                (-1, -2), // T spins
                ( 1, -2),
            ];

            for try_move in try_moves.iter() {
                for (_entity, mut position, _tetromino, _current) in current_query.iter_mut() {
                    position.x += try_move.0;
                    position.y += try_move.1;
                }

                if check_tetromino_positions(&mut current_query, &mut heap_query) {
                    should_revert = false;
                    break;
                }
            }
        } else {
            // Revert movement and add to heap
            for (entity, _position, _tetromino, _current) in current_query.iter_mut() {
                commands.entity(entity)
                    .remove::<CurrentTetromino>()
                    .insert(Heap)
                    ;
                //commands.remove_one::<CurrentTetromino>(entity);
                //commands.insert_one(entity, Heap);
            }
            //clear_full_layers(commands, &mut heap_query);
            for matrix in matrix_query.iter_mut() {
                spawn_current_tetromino(&mut commands, matrix, &mut materials);
            }
            
        }

        if should_revert {
            for (entity, mut position, _tetromino, _current) in current_query.iter_mut() {
                let prev_position = prev_positions.get(&entity.id()).unwrap();
                position.x = prev_position.0;
                position.y = prev_position.1;
            }
        }
    }
}

fn update_block_sprites(
    mut matrix_query: Query<(&Matrix, &Sprite)>,
    mut block_query: Query<(&MatrixPosition, &mut Transform)>,
    time: Res<Time>,
    mut updatetimer: ResMut<UpdateTimer>
) {
    updatetimer.0.tick(time.delta());
    for (_matrix, matrix_sprite) in matrix_query.iter_mut() {
        for (position, mut transform) in block_query.iter_mut() {
//             let new_x: f32 = ((position.x as f32 * Block::SIZE) - (matrix_sprite.custom_size.x * 0.5)) + (Block::SIZE * 0.5);
//             let new_y: f32 = (matrix_sprite.size.y * -0.5) + (position.y as f32 * Block::SIZE) + (Block::SIZE * 0.5);
            let new_x: f32 = ((position.x as f32 * Block::SIZE) - (matrix_sprite.custom_size.unwrap().x * 0.5)) + (Block::SIZE * 0.5);
            let new_y: f32 = (matrix_sprite.custom_size.unwrap().y * -0.5) + (position.y as f32 * Block::SIZE) + (Block::SIZE * 0.5);

            let translation = &mut transform.translation;
            translation.x = new_x;
            translation.y = new_y;
        }
    }
}

// ----------------
// UTILITY AND IMPL
// ----------------

fn clear_full_layers(
    mut commands: Commands,
    time: Res<Time>,
    mut updatetimer: ResMut<UpdateTimer>,
    mut heap_query: Query<(Entity, &mut MatrixPosition, &Heap)>
    ) {
    let width: i32 = 10;
    let height: i32 = 22;
    //bug, called with old heap rather than one given as command
    let mut cond: bool = false;
    updatetimer.0.tick(time.delta());
    for y in 0..height {
        for x in 0..width {
            cond = false;
            for (_ent, heap_position, _heap) in heap_query.iter_mut() {
                if heap_position.x==x && heap_position.y==y {
                
                    cond = true;
                    break;
                }
            }
            if cond == false {break;}
        }
        if cond == true {
            println!("layer {} completed", y);
            for (ent, mut heap_position, _heap) in heap_query.iter_mut() {
                if heap_position.y==y {
                    commands.entity(ent).despawn();
                } else if heap_position.y > y {
                    heap_position.y -= 1;
                }
            }
        }
    }
}

fn rotate_tetromino_block(tetromino_block: &mut Tetromino, matrix_size: i32, clockwise: bool) {
    let orig_x = tetromino_block.index.x;
    let orig_y = tetromino_block.index.y;
    let matrix_size = matrix_size - 1;

    let x = orig_x;
    if clockwise {
        tetromino_block.index.x = orig_y;
        tetromino_block.index.y = matrix_size - x;
    } else {
        tetromino_block.index.x = matrix_size - orig_y;
        tetromino_block.index.y = orig_x;
    }
}

fn check_tetromino_positions(
    current_query: &mut Query<(Entity, &mut MatrixPosition, &mut Tetromino, &CurrentTetromino), Without<Heap>>,
    heap_query: &mut Query<(Entity, &mut MatrixPosition, &Heap)>
) -> bool {
    for (_entity, position, _tetromino, _current) in current_query.iter_mut() {
        if position.y < 0 {
            return false;
        }

        for (_ent, heap_position, _heap) in heap_query.iter_mut() {
            if position.x == heap_position.x && position.y == heap_position.y {
                return false;
            }
        }
    }

    return true;
}

fn spawn_current_tetromino(
    commands: &mut Commands,
    matrix: &Matrix,
    _materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let blocks = Tetromino::blocks_from_type(rand::random());
    for block in blocks.into_iter() {
        let tetromino_matrix_size = Tetromino::SIZES[block.1.tetromino_type as usize];
        
        let mysprite = Sprite {
            color: Color::rgb(
                    block.0.color.r(),
                    block.0.color.g(),
                    block.0.color.b()
                    ).into(),
            flip_x: false,
            flip_y: false,
            custom_size: Some(Vec2::new(Block::SIZE, Block::SIZE)),
        };
        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                sprite: mysprite,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..Default::default()
            })
            .insert(CurrentTetromino)
            .insert(MatrixPosition {
                x: block.1.index.x + 3,
                y: matrix.height - tetromino_matrix_size + block.1.index.y,
            })
            .insert_bundle(block)
        ;
    }
}

#[derive(Copy, Clone, Debug)]
enum TetrominoType {
    I = 0,
    O = 1,
    T = 2,
    S = 3,
    Z = 4,
    L = 5,
    J = 6,
}

impl Tetromino {
    const BLOCK_INDICES: [[(i32, i32); 4]; 7] = [
        [ // line, cyan
            (1, 3),
            (1, 2),
            (1, 1),
            (1, 0),
        ],
        [ // square, yellow
            (1, 1),
            (1, 2),
            (2, 1),
            (2, 2),
        ],
        [ // T, purple
            (0, 1),
            (1, 1),
            (2, 1),
            (1, 2),
        ],
        [ // Z, red
            (0, 2),
            (1, 2),
            (1, 1),
            (2, 1),
        ],
        [ // S, green
            (2, 2),
            (1, 2),
            (1, 1),
            (0, 1),
        ],
        [ // L, blue
            (0, 2),
            (0, 1),
            (1, 1),
            (2, 1),
        ],
        [ // J, orange
            (0, 1),
            (1, 1),
            (2, 1),
            (2, 2),
        ],
    ];

    const COLORS: [(f32, f32, f32); 7] = [
        (0.0, 0.7, 0.7), // line, cyan
        (0.7, 0.7, 0.0), // square, yellow
        (0.7, 0.0, 0.7), // T, purple
        (0.7, 0.0, 0.0), // Z, red
        (0.0, 0.7, 0.0), // S, green
        (0.0, 0.0, 0.7), // L, blue
        (0.9, 0.25, 0.0), // J, orange
    ];

    const SIZES: [i32; 7] = [
        4, // line, cyan
        4, // square, yellow
        3, // T, purple
        3, // Z, red
        3, // S, green
        3, // L, blue
        3, // J, orange
    ];

    fn blocks_from_type(tetromino_type: TetrominoType) -> Vec<(Block, Tetromino)> {
        let type_usize = tetromino_type as usize;
        let color = Tetromino::COLORS[type_usize];

        Tetromino::BLOCK_INDICES[type_usize].iter()
            .map(|index| {
                (
                    Block {
                        color: Color::rgb(color.0, color.1, color.2),
                    },
                    Tetromino {
                        index: MatrixPosition {
                            x: index.0,
                            y: index.1,
                        },
                        tetromino_type
                    }
                )
            })
            .collect()
    }
}

impl Distribution<TetrominoType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TetrominoType {
        match rng.gen_range(0, 7) {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::L,
            _ => TetrominoType::J
        }
    }
}
