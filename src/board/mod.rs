use std::ops;

use super::{PlayerId, TerritoryId, NumArmies};
pub use self::standard_board::StandardGameBoard;

mod standard_board;

#[derive(Copy, Clone, Debug)]
pub enum Continent {
    Australia,
    SouthAmerica,
    Africa,
    Europe,
    NorthAmerica,
    Asia,
}

impl Continent {
    fn get_range(&self) -> ops::Range<u8> {
        match *self {
            Continent::Africa        => 0..6,
            Continent::Asia          => 6..18,
            Continent::Australia     => 18..22,
            Continent::Europe        => 22..29,
            Continent::NorthAmerica  => 29..38,
            Continent::SouthAmerica  => 38..42
        }
    }

    fn get_bonus(&self) -> u8 {
        match *self {
            Continent::Australia     => 2,
            Continent::SouthAmerica => 2,
            Continent::Africa        => 3,
            Continent::Europe        => 5,
            Continent::NorthAmerica => 5,
            Continent::Asia          => 7,
        }
    }
}

// Game board models the state of board:
//
//    a) the owner of each territory
//    b) how many occupying armies on each territory
//
// The data structure representing the game board should supply
// methods for changing the board state.

pub trait GameBoard {
    fn get_owner(&self, TerritoryId) -> PlayerId;
    fn get_num_armies(&self, TerritoryId) -> NumArmies;
    fn get_num_owned_territories(&self, PlayerId) -> u8;
    fn get_owned_territories(&self, PlayerId) -> Vec<TerritoryId>;
    fn get_continent_bonuses(&self, PlayerId) -> u8;
    fn player_owns_continent(&self, PlayerId, Continent) -> bool;

    // calculate to total number of reinforcements that a player will
    // receive from terrritories held and continent bonuses
    fn get_territory_reinforcements(&self, player: PlayerId) -> NumArmies;

    fn set_territory(&mut self, TerritoryId, PlayerId, NumArmies);
    fn game_is_over(&self) -> bool;
    fn player_is_defeated(&self, PlayerId) -> bool;

    // A GameBoard has an underlying GameMap
    fn game_map(&self) -> &GameMap;

    // TODO: this should open a window, I guess? do we need &mut self for this?
    fn display(&self);

    fn is_enemy_territory(&self, player: PlayerId, tid: TerritoryId) -> bool {
        self.get_owner(tid) != player
    }

    fn add_armies(&mut self, tid: TerritoryId, add: NumArmies) {
        let num_armies = self.get_num_armies(tid);
        let owner = self.get_owner(tid);
        self.set_territory(tid, owner, num_armies + add);
    }

    fn remove_armies(&mut self, tid: TerritoryId, remove: NumArmies) {
        let num_armies = self.get_num_armies(tid);

        if remove > num_armies {
            panic!("Cannot remove more armies than exist on territory {}", tid);
        }

        let owner = self.get_owner(tid);
        self.set_territory(tid, owner, num_armies - remove);
    }
}

pub trait GameMap {
    fn are_adjacent(&self, a: TerritoryId, b: TerritoryId) -> bool;
    fn get_neighbors(&self, TerritoryId) -> Vec<TerritoryId>;
}
