use petgraph::{Graph, Undirected};
use petgraph::graph::NodeIndex;
use rand::{self, Rng};
use std::ops;

use super::{PlayerId, TerritoryId, NUM_TERRITORIES};

// Game board: contains publically available game state
//
//    a) the owner of each territory
//    b) how many occupying armies on each territory
//    c) how many risk cards each player has
//
// The data structure representing the game board should supply
// methods for changing the board state.

pub trait GameBoard {
    fn get_owner(&self, TerritoryId) -> PlayerId;
    fn get_num_armies(&self, TerritoryId) -> u8;
    fn get_num_cards(&self, PlayerId) -> u8;
    fn get_num_owned_territories(&self, PlayerId) -> u8;
    fn get_owned_territories(&self, PlayerId) -> Vec<TerritoryId>;
    fn get_continent_bonuses(&self, PlayerId) -> u8;
    fn player_owns_continent(&self, PlayerId, Continent) -> bool;

    // calculate to total number of reinforcements that a player will
    // receive from terrritories held and continent bonuses
    fn get_territory_reinforcements(&self, player: PlayerId) -> u8;

    fn set_territory(&mut self, TerritoryId, PlayerId, u8);
    fn set_num_cards(&mut self, PlayerId, u8);
    fn game_is_over(&self) -> bool;
    fn player_is_defeated(&self, PlayerId) -> bool;

    // A GameBoard has an underlying GameMap
    fn game_map(&self) -> &GameMap;
}

pub trait GameMap {
    fn are_adjacent(&self, a: TerritoryId, b: TerritoryId) -> bool;
    fn get_neighbors(&self, TerritoryId) -> Vec<TerritoryId>;
}

pub type TerritoryGraph = Graph<(PlayerId, u8), (), Undirected, TerritoryId>;

impl GameMap for TerritoryGraph {
    fn are_adjacent(&self, a: TerritoryId, b: TerritoryId) -> bool {
        for neighbor in self.neighbors(NodeIndex::new(a as usize)) {
            if neighbor == NodeIndex::new(b as usize) {
                return true;
            }
        }
        false
    }

    fn get_neighbors(&self, t: TerritoryId) -> Vec<TerritoryId> {
        let mut neighbors = Vec::new();

        for n in self.neighbors(NodeIndex::new(t as usize)) {
            neighbors.push(n.index() as TerritoryId);
        }
        neighbors
    }
}

pub fn standard_risk_map() -> TerritoryGraph {
    // TODO: correct number of edges
    let mut graph = TerritoryGraph::with_capacity(42, 100);
    // let mut graph = Graph::new_undirected();
    graph
}


#[derive(Copy, Clone)]
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
            Continent::Australia     => 0..4,
            Continent::SouthAmerica => 4..8,
            Continent::Africa        => 8..14,
            Continent::Europe        => 14..21,
            Continent::NorthAmerica => 21..30,
            Continent::Asia          => 30..42,
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

pub type GameBoardTerritories = [(PlayerId, u8); NUM_TERRITORIES];

// a standard Risk gameboard has 42 territories
pub struct StandardGameBoard {
    num_players: u8,
    territories: GameBoardTerritories,
    num_cards: Vec<u8>,
    map: TerritoryGraph,
}

impl StandardGameBoard {
    pub fn new(num_players: u8, territories: GameBoardTerritories) -> StandardGameBoard {
        StandardGameBoard {
            num_players: num_players,
            territories: territories,
            num_cards: vec![0; num_players as usize],
            map: standard_risk_map(),
        }
    }
    
    pub fn randomly_distributed(num_players: u8) -> StandardGameBoard {
        StandardGameBoard::new(num_players,
                               StandardGameBoard::distrib_terr_randomly(num_players))
    }

    // distributes the territories as equally as possible among the available players
    fn distrib_terr_randomly(num_players: u8) -> GameBoardTerritories {
        let mut territories = [(0, 1); NUM_TERRITORIES];
        let mut player_pool: Vec<_> = (0..num_players).collect();
        for i in 0..NUM_TERRITORIES {
            if player_pool.len() == 0 {
                player_pool = (0..num_players).collect();
            }

            let rand_player = rand::thread_rng().gen_range(0, player_pool.len());
            territories[i].0 = player_pool[rand_player];
            player_pool.remove(rand_player);
            println!("owner of {} is {}", i, territories[i].0);
        }
        territories
    }
}



impl GameBoard for StandardGameBoard {
    fn get_owner(&self, terr: TerritoryId) -> PlayerId {
        self.territories[terr as usize].0
    }

    fn get_num_armies(&self, terr: TerritoryId) -> u8 {
        self.territories[terr as usize].1
    }

    fn get_num_cards(&self, player: PlayerId) -> u8 {
        self.num_cards[player as usize]
    }

    fn get_num_owned_territories(&self, player: PlayerId) -> u8 {
        let mut count = 0;
        for i in 0..NUM_TERRITORIES {
            if player == self.territories[i].0 {
                count += 1;
            }
        }
        count
    }

    fn get_owned_territories(&self, player: PlayerId) -> Vec<TerritoryId> {
        let mut terrs = vec![];
        for i in 0..NUM_TERRITORIES {
            if player == self.territories[i].0 {
                terrs.push(i as TerritoryId);
            }
        }
        terrs
    }

    fn get_continent_bonuses(&self, player: PlayerId) -> u8 {
        let mut bonus = 0;

        let continents = [Continent::Australia,
                          Continent::SouthAmerica,
                          Continent::Africa,
                          Continent::Europe,
                          Continent::NorthAmerica,
                          Continent::Asia];

        for continent in continents.iter() {
            if self.player_owns_continent(player, *continent) {
                bonus += continent.get_bonus();
            }
        }

        bonus
    }

    fn player_owns_continent(&self, player: PlayerId, continent: Continent) -> bool {
        for i in continent.get_range() {
            if self.get_owner(i) != player {
                return false;
            }
            println!("player owns {}", i);
        }
        true
    }

    fn get_territory_reinforcements(&self, player: PlayerId) -> u8 {
        let num_terr = self.get_num_owned_territories(player);
        let continent_bonuses = self.get_continent_bonuses(player);
        num_terr + continent_bonuses
    }

    fn set_territory(&mut self, terr: TerritoryId, owner: PlayerId, num_armies: u8) {
        if owner as u8 > self.num_players {
            panic!("Error in `set_armies`: invalid player for owner");
        }

        self.territories[terr as usize] = (owner, num_armies);
    }

    fn set_num_cards(&mut self, player: PlayerId, num_cards: u8) {
        if player as u8 > self.num_players {
            panic!("Error in `set_num_cards`: invalid player");
        }

        self.num_cards[player as usize] = num_cards;
    }

    fn game_is_over(&self) -> bool {
        let owner0 = self.get_owner(0);
        for i in 1..NUM_TERRITORIES {
            if self.get_owner(i as TerritoryId) != owner0 {
                return false;
            }
        }
        true
    }

    fn player_is_defeated(&self, player: PlayerId) -> bool {
        for i in 0..NUM_TERRITORIES {
            if player == self.territories[i].0 {
                return false;
            }
        }
        true
    }

    fn game_map(&self) -> &GameMap {
        &self.map
    }
}

