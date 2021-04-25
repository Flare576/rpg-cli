extern crate dirs;

use crate::location::Location;
use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub player: Player,
    pub location: Location,
}

// TODO factor out all dir/file management code
fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}

impl Game {
    pub fn load() -> Result<Self, io::Error> {
        match fs::read(data_file()) {
            Ok(data) => {
                let game: Game = bincode::deserialize(&data).unwrap();
                Ok(game)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::new()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let rpg_dir = rpg_dir();
        if !rpg_dir.exists() {
            fs::create_dir(&rpg_dir).unwrap();
        }

        let data = bincode::serialize(&self).unwrap();
        fs::write(data_file(), &data)
    }

    pub fn new() -> Self {
        Self {
            location: Location::home(),
            player: Player::new(),
        }
    }

    pub fn walk_towards(&mut self, dest: &Location) {
        while self.location != *dest {
            self.location.walk_towards(&dest);
            // print current location
            // if home -> heal
            // else if let Some(enemy) = maybe_spawn_enemy()
            // battle + return
        }
    }
}
