use std::convert::From;
use vindinium::{Dir, State};
use pathing;
use pathing::{UVector2, IVector2, Map};

fn find_destination(state: &State) -> Option<UVector2> {
    None
}

fn get_direction(pos1: &UVector2, pos2: &UVector2) -> Dir {
    let cur_pos = IVector2::from(pos1);
    let new_pos = IVector2::from(pos2);

    return match (cur_pos.x - new_pos.x, cur_pos.y - new_pos.y) {
        (0,1) => Dir::North,
        (1,0) => Dir::East,
        (0,-1) => Dir::South,
        (-1,0) => Dir::West,
        (_,_) => {
            println!("Could not determine direction returned from path!");
            Dir::Stay
        }
    };
}

pub fn step(state: &State) -> Dir {
    let cur_pos = UVector2{x: state.hero.pos.x as usize, y: state.hero.pos.y as usize};
    let w_destination = find_destination(state);
    let dir = match w_destination {
        Some(dest) => {
            let w_path = pathing::gen_path(
                &cur_pos, &dest, &Map::from(&state.game.board)
            );
            match w_path {
                Some(path) => get_direction(&cur_pos, &path.front().unwrap().pos),
                None => Dir::Stay //Path from bot pos to target is not valid TODO: Find a new target?
            }
        }
        None => {
            println!("ERROR: Could not determine a destination.");
            Dir::Stay
        }
    };

    dir
}
