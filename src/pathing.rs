extern crate term;
use vindinium;
use vindinium::{Tile, Board};
use std::convert::From;
use std::collections::LinkedList;
use self::term::{Terminal};
use self::term::color;

pub type Grid = Vec<Vec<Cell>>;
pub type Path = LinkedList<Cell>;

#[derive(Debug, PartialEq, Clone)]
pub struct IVector2 {
    pub x: isize,
    pub y: isize
}

#[derive(Debug, PartialEq, Clone)]
pub struct UVector2 {
    pub x: usize,
    pub y: usize
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cell {
    pub tile: Tile,
    pub pos: UVector2,
    pub f: u8,
    pub g: u8,
    pub h: u8
}

#[derive(Debug, Clone)]
pub struct Map {
    pub size: u8,
    pub grid: Grid
}

impl<'a> From<&'a Board> for Map {
    fn from(board: &'a Board) -> Self {
        let tiles = &board.tiles;
        let mut cells: Grid = Grid::new();

        for x in 0..board.size {
            let mut new_row: Vec<Cell> = Vec::new();
            for y in 0..board.size {
                new_row.push(Cell {tile: tiles[x][y].clone(), pos: UVector2 {x: x, y: y}, f: 0, g: 0, h: 0});
            }
            cells.push(new_row);
        }

        Map {size: board.size as u8, grid: cells}
    }
}

impl<'a> From<&'a UVector2> for IVector2 {
    fn from(vec: &'a UVector2) -> Self {
        IVector2 {x: vec.x as isize, y: vec.y as isize}
    }
}

impl<'a> From<&'a IVector2> for UVector2 {
    fn from(vec: &'a IVector2) -> Self {
        let x = vec.x.abs();
        let y = vec.y.abs();
        UVector2 {x: x as usize, y: y as usize}
    }
}

pub fn print_over(path: &Path, map: &Map) {
    let mut term = term::stdout().unwrap();

    // print tiles and path on board
    for (x, row) in map.grid.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            let p_cell = path.iter().filter(|p_cell| p_cell.pos == map.grid[x][y].pos).next();
            let s: String = match p_cell {
                Some(x) => {
                    term.bg(color::BRIGHT_BLACK).unwrap();
                    term.fg(color::WHITE).unwrap();
                    "><".to_string()
                },
                    None => vindinium::get_tile_rep(&map.grid[x][y].tile, &mut term)
            };
            (write!(term, "{}", s)).unwrap();
        }
        (writeln!(term,"")).unwrap();
    }
}

fn calc_neighbor(ref_cell: &Cell, target_pos: &UVector2, diag: bool) -> Cell {
    let mut cell = ref_cell.clone();
    cell.g = match diag {
        true => 40_u8, // cannot move diagonally in this game, hence the large value
        false => 10_u8
    };
    let tp: IVector2 = IVector2::from(target_pos);

    // calculate rough manhattan distance from target
    let cp: IVector2 = IVector2::from(&cell.pos);
    let h = 10_isize*((cp.x-tp.x).abs()) + (cp.y-tp.y).abs();

    assert!(h <= 255);
    cell.h = h as u8;

    // sum score
    cell.f = cell.g + cell.h;

    if cell.tile != Tile::Free {
       cell.f = 255_u8; // cannot move into space!
    }

    cell
}

/// Calculates f, g, and h values for each cell surrounding the `cp` parameter.
/// - Returns surrounding neighbors with calculated values
fn calc_neighbors(cp: &UVector2, target_pos: &UVector2, cells: &Grid, grid_size: usize) -> Vec<Cell> {
    // ul-uv-ur
    // hl-cp-hr
    // dl-dv-dr
    let i_grid_size = grid_size as isize;
    let i_cp = IVector2::from(cp);
    let mut open_cells: Vec<Cell> = vec!();

    // quick constrain bounds
    println!("x: {} y: {}", cp.x, cp.y);
    if (i_cp.x-1 >= 0 && i_cp.y-1 >= 0) && (i_cp.x+1 < i_grid_size && i_cp.y+1 < i_grid_size) {
        // now safe to not do bounds checking
        // diagonal d-weights
        open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y+1], target_pos, true)); // ul
        open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y-1], target_pos, true)); // dl
        open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y+1], target_pos, true)); // ur
        open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y-1], target_pos, true)); // dr
        // end diagonal d-weights
        open_cells.push(calc_neighbor(&cells[cp.x][cp.y+1], target_pos, false)); // uv
        open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y], target_pos, false)); // hr
        open_cells.push(calc_neighbor(&cells[cp.x][cp.y-1], target_pos, false)); // dv
        open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y], target_pos, false)); // dl
    }
    // slow constrain bounds :(
    else {
        open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y], target_pos, false));      // hl
        open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y-1], target_pos, true));     // dl
        open_cells.push(calc_neighbor(&cells[cp.x][cp.y-1], target_pos, false));      // dv

        if cp.y < grid_size-1 {
            open_cells.push(calc_neighbor(&cells[cp.x-1][cp.y+1], target_pos, true)); // ul
            open_cells.push(calc_neighbor(&cells[cp.x][cp.y+1], target_pos, false));  // uv
        }

        if cp.x < grid_size - 1 {
            open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y], target_pos, false));      // hr

            if cp.y < grid_size-1 {
                open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y+1], target_pos, true)); // ur
            }

            open_cells.push(calc_neighbor(&cells[cp.x+1][cp.y-1], target_pos, true)); // dr
        }
    }
    // end constrain bounds
    open_cells
}

pub fn gen_path(bot_pos: &UVector2, target_pos: &UVector2, map: &Map) -> Path {
    let path_grid = &map.grid;
    //let ref target_cell = path_cells[target_pos.x][target_pos.y];
    let mut open_cells: Vec<Cell> = vec!((path_grid[bot_pos.x][bot_pos.y].clone()));
    let mut closed_cells: LinkedList<Cell> = LinkedList::new();
    let tp = target_pos.clone();

    // gather optimal f-val cells
    loop {
        match closed_cells.back() {
            Some(cell) if cell.pos == tp => {break;},
            _ => {}
        };

        if open_cells.len() > 0 {
            let mut best_ind = 0;
            for i in 0..open_cells.len() {
                if open_cells[i].f <= open_cells[best_ind].f { // cur_cell.f < best_cell.f
                    best_ind = i;
                }
            }

            // calculate and get neighbors to current cell
            let ref mut neighbors = calc_neighbors(&open_cells[best_ind].pos, &target_pos, &path_grid, map.size as usize);

            // pop most optimal cell off of open cells and add to closed cells
            let best_cell = open_cells.remove(best_ind);
            closed_cells.push_back(best_cell);

            // Remove new neighbors if it is already in the closed list
            for ref cell in &closed_cells {
                let mut ind: isize = -1;
                for i in 0..neighbors.len() {
                    let ref other = neighbors[i];
                    if cell.pos == other.pos {
                        ind = i as isize;
                        break;
                    }
                }

                if ind >= 0 {
                    neighbors.remove(ind as usize);
                }
            }

            // Remove new neighbors if already in the open list--update open cell if g val is better
            for mut cell in &mut open_cells {
                let mut ind: isize = -1;
                for i in 0..neighbors.len() {
                    let ref other = neighbors[i];
                    if cell.pos == other.pos {
                        ind = i as isize;
                        if cell.g > other.g {
                            cell.f = other.f;
                            cell.g = other.g;
                            cell.h = other.h;
                        }
                        break;
                    }
                }

                if ind >= 0 {
                    neighbors.remove(ind as usize);
                }
            }

            // append new neighbors
            open_cells.append(neighbors);
        }
        else {
            panic!("No more cells to calculate!? Path not found");
        }
    }

    // determine path
    closed_cells.pop_front();
    closed_cells
}