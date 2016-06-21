use petgraph::{Graph, Undirected};
use petgraph::graph::NodeIndex;
use rand::{self, Rng};
use std::ops;

use super::{PlayerId, TerritoryId, NumArmies, NUM_TERRITORIES};

// Game board: contains publically available game state
//
//    a) the owner of each territory
//    b) how many occupying armies on each territory
//    c) how many cards each player has
//
// The data structure representing the game board should supply
// methods for changing the board state.

pub trait GameBoard {
    fn get_owner(&self, TerritoryId) -> PlayerId;
    fn get_num_armies(&self, TerritoryId) -> NumArmies;
    fn get_num_cards(&self, PlayerId) -> u8;
    fn get_num_owned_territories(&self, PlayerId) -> u8;
    fn get_owned_territories(&self, PlayerId) -> Vec<TerritoryId>;
    fn get_continent_bonuses(&self, PlayerId) -> u8;
    fn player_owns_continent(&self, PlayerId, Continent) -> bool;

    // calculate to total number of reinforcements that a player will
    // receive from terrritories held and continent bonuses
    fn get_territory_reinforcements(&self, player: PlayerId) -> NumArmies;

    fn set_territory(&mut self, TerritoryId, PlayerId, NumArmies);
    fn set_num_cards(&mut self, PlayerId, u8);
    fn game_is_over(&self) -> bool;
    fn player_is_defeated(&self, PlayerId) -> bool;

    // A GameBoard has an underlying GameMap
    fn game_map(&self) -> &GameMap;

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

pub type TerritoryGraph = Graph<(), (), Undirected, TerritoryId>;

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

pub fn standard_map() -> TerritoryGraph {
    // TODO: correct number of edges
    let mut graph = TerritoryGraph::with_capacity(42, 100);
    let mut indices = Vec::new();
    for _ in 0..(NUM_TERRITORIES as TerritoryId) {
        indices.push( graph.add_node(()) );
    }

    for i in 0..(NUM_TERRITORIES as TerritoryId) {
        for &n in StandardTerritory::from_territory_id(i).neighbors().iter() {
            graph.add_edge(indices[i as usize], indices[n as usize], ());
        }
    }
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

#[derive(Copy, Clone)]
enum StandardTerritory {
    Congo = 0,
    EastAfrica = 1,
    Egypt = 2,
    Madagascar = 3,
    NorthAfrica = 4,
    SouthAfrica = 5,

    Afghanistan = 6,
    China = 7,
    India = 8,
    Irkutsk = 9,
    Japan = 10,
    Kamchatka = 11,
    MiddleEast = 12,
    Mongolia = 13,
    Siam = 14,
    Siberia = 15,
    Ural = 16,
    Yakutsk = 17,

    EasternAustralia = 18,
    Indonesia = 19,
    NewGuinea = 20,
    WesternAustralia = 21,

    GreatBritain = 22,
    Iceland = 23,
    NorthernEurope = 24,
    Scandinavia = 25,
    SouthernEurope = 26,
    Ukraine = 27,
    WesternEurope = 28,

    Alaska = 29,
    Alberta = 30,
    CentralAmerica = 31,
    EasternUS = 32,
    Greenland = 33,
    NorthwestTerritory = 34,
    Ontario = 35,
    Quebec = 36,
    WesternUS = 37,

    Argentina = 38,
    Brazil = 39,
    Peru = 40,
    Venezuela = 41,
}

impl StandardTerritory {
    fn from_territory_id(tid: TerritoryId) -> StandardTerritory {
        use self::StandardTerritory::*;
        match tid {
            0 => Congo,
            1 => EastAfrica,
            2 => Egypt,
            3 => Madagascar,
            4 => NorthAfrica,
            5 => SouthAfrica,

            6 => Afghanistan,
            7 => China,
            8 => India,
            9 => Irkutsk,
            10 => Japan,
            11 => Kamchatka,
            12 => MiddleEast,
            13 => Mongolia,
            14 => Siam,
            15 => Siberia,
            16 => Ural,
            17 => Yakutsk,

            18 => EasternAustralia,
            19 => Indonesia,
            20 => NewGuinea,
            21 => WesternAustralia,

            22 => GreatBritain,
            23 => Iceland,
            24 => NorthernEurope,
            25 => Scandinavia,
            26 => SouthernEurope,
            27 => Ukraine,
            28 => WesternEurope,

            29 => Alaska,
            30 => Alberta,
            31 => CentralAmerica,
            32 => EasternUS,
            33 => Greenland,
            34 => NorthwestTerritory,
            35 => Ontario,
            36 => Quebec,
            37 => WesternUS,

            38 => Argentina,
            39 => Brazil,
            40 => Peru,
            41 => Venezuela,
            _ => panic!("Territory ID out of bounds for StandardTerritory"),
        }
    }

    fn neighbors(&self) -> Vec<StandardTerritory> {
        use self::StandardTerritory::*;
        match *self {
            Congo => vec![EastAfrica, NorthAfrica, SouthAfrica],
            EastAfrica => vec![Congo, Egypt, Madagascar, NorthAfrica, MiddleEast],
            Egypt => vec![EastAfrica, NorthAfrica, SouthernEurope],
            Madagascar => vec![EastAfrica, SouthAfrica],
            NorthAfrica => vec![Congo, EastAfrica, Egypt, SouthernEurope, WesternEurope, Brazil],
            SouthAfrica => vec![Congo, EastAfrica, Madagascar],

            Afghanistan => vec![China, India, MiddleEast, Ural, Ukraine],
            China => vec![Afghanistan, India, Mongolia, Siam, Siberia, Ural],
            India => vec![Afghanistan, China, MiddleEast, Siam],
            Irkutsk => vec![Kamchatka, Mongolia, Siberia, Yakutsk],
            Japan => vec![Kamchatka, Mongolia],
            Kamchatka => vec![Irkutsk, Japan, Yakutsk, Alaska],
            MiddleEast => vec![EastAfrica, Egypt, Afghanistan, India, SouthernEurope, Ukraine],
            Mongolia => vec![China, Irkutsk, Japan, Kamchatka, Siberia],
            Siam => vec![China, India, Indonesia],
            Siberia => vec![China, Irkutsk, Mongolia, Ural, Yakutsk],
            Ural => vec![Afghanistan, China, Siberia, Ukraine],
            Yakutsk => vec![Irkutsk, Kamchatka, Siberia],

            EasternAustralia => vec![NewGuinea, WesternAustralia],
            Indonesia => vec![Siam, NewGuinea, WesternAustralia],
            NewGuinea => vec![EasternAustralia, Indonesia, WesternAustralia],
            WesternAustralia => vec![EasternAustralia, Indonesia, NewGuinea],

            GreatBritain => vec![Iceland, NorthernEurope, Scandinavia, WesternEurope],
            Iceland => vec![GreatBritain, Scandinavia, Greenland],
            NorthernEurope => vec![GreatBritain, Scandinavia, SouthernEurope, Ukraine, WesternEurope],
            Scandinavia => vec![GreatBritain, Iceland, NorthernEurope, Ukraine],
            SouthernEurope => vec![Egypt, NorthAfrica, MiddleEast, NorthernEurope, Ukraine, WesternEurope],
            Ukraine => vec![Afghanistan, MiddleEast, Ural, NorthernEurope, Scandinavia, SouthernEurope],
            WesternEurope => vec![NorthAfrica, GreatBritain, NorthernEurope, SouthernEurope],

            Alaska => vec![Kamchatka, Alberta, NorthwestTerritory],
            Alberta => vec![Alaska, NorthwestTerritory, Ontario, WesternUS],
            CentralAmerica => vec![EasternUS, WesternUS, Venezuela],
            EasternUS => vec![CentralAmerica, Ontario, Quebec, WesternUS],
            Greenland => vec![Iceland, NorthwestTerritory, Ontario, Quebec],
            NorthwestTerritory => vec![Alaska, Alberta, Greenland, Ontario],
            Ontario => vec![Alberta, EasternUS, Greenland, NorthwestTerritory, Quebec, WesternUS],
            Quebec => vec![EasternUS, Greenland, Ontario],
            WesternUS => vec![Alberta, CentralAmerica, EasternUS, Ontario],

            Argentina => vec![Brazil, Peru,],
            Brazil => vec![NorthAfrica, Argentina, Peru, Venezuela],
            Peru => vec![Argentina, Brazil, Venezuela],
            Venezuela => vec![CentralAmerica, Brazil, Peru],
        }
    }
}


pub type GameBoardTerritories = [(PlayerId, NumArmies); NUM_TERRITORIES];

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
            map: standard_map(),
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

    fn get_num_armies(&self, terr: TerritoryId) -> NumArmies {
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

    fn get_territory_reinforcements(&self, player: PlayerId) -> NumArmies {
        use std::cmp::max;
        let num_terr = self.get_num_owned_territories(player);
        let continent_bonuses = self.get_continent_bonuses(player);
        (max((num_terr % 3), 3) + continent_bonuses) as NumArmies
    }

    fn set_territory(&mut self, terr: TerritoryId, owner: PlayerId, num_armies: NumArmies) {
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
