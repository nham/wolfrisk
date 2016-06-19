extern crate rand;

use rand::Rng;

const NUM_TERRITORIES: usize = 42;

#[derive(Copy, Clone)]
enum Continent {
    Australia,
    South_America,
    Africa,
    Europe,
    North_America,
    Asia,
}

impl Continent {
    fn get_range(&self) -> std::ops::Range<u8> {
        match *self {
            Continent::Australia     => 0..4,
            Continent::South_America => 4..8,
            Continent::Africa        => 8..14,
            Continent::Europe        => 14..21,
            Continent::North_America => 21..30,
            Continent::Asia          => 30..42,
        }
    }

    fn get_bonus(&self) -> u8 {
        match *self {
            Continent::Australia     => 2,
            Continent::South_America => 2,
            Continent::Africa        => 3,
            Continent::Europe        => 5,
            Continent::North_America => 5,
            Continent::Asia          => 7,
        }
    }
}

// Loosely follow architecture from Wolf's thesis
// Game manager: contains a game board, some rules, and a collection of players?
//
//    a)

// Game board: contains publically available game state
//
//    a) the owner of each territory
//    b) how many occupying armies on each territory
//    c) how many risk cards each player has
//
// The data structure representing the game board should supply
// methods for changing the board state.

type TerritoryId = u8;
type PlayerId = u8;

trait GameBoard {
    fn get_owner(&self, TerritoryId) -> PlayerId;
    fn get_num_armies(&self, TerritoryId) -> u8;
    fn get_num_cards(&self, PlayerId) -> u8;
    fn get_num_owned_territories(&self, PlayerId) -> u8;
    fn get_continent_bonuses(&self, PlayerId) -> u8;
    fn player_owns_continent(&self, PlayerId, Continent) -> bool;

    // calculate to total number of reinforcements that a player will
    // receive from terrritories held and continent bonuses
    fn get_territory_reinforcements(&self, player: PlayerId) -> u8;

    fn set_territory(&mut self, TerritoryId, PlayerId, u8);
    fn set_num_cards(&mut self, PlayerId, u8);
    fn game_is_over(&self) -> bool;
    fn player_is_defeated(&self, PlayerId) -> bool;
}

type GameBoardTerritories = [(PlayerId, u8); NUM_TERRITORIES];

// a standard Risk gameboard has 42 territories
struct StandardGameBoard {
    num_players: u8,
    territories: GameBoardTerritories,
    num_cards: Vec<u8>,
}

impl StandardGameBoard {
    fn new(num_players: u8, territories: GameBoardTerritories) -> StandardGameBoard {
        StandardGameBoard {
            num_players: num_players,
            territories: territories,
            num_cards: vec![0; num_players as usize],
        }
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

    fn get_continent_bonuses(&self, player: PlayerId) -> u8 {
        let mut bonus = 0;

        let continents = [Continent::Australia,
                          Continent::South_America,
                          Continent::Africa,
                          Continent::Europe,
                          Continent::North_America,
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
}

#[derive(Copy, Clone)]
struct Trade {
    cards: [Card; 3],
}

impl Trade {
    fn new(cards: [Card; 3]) -> Trade {
        Trade { cards: cards }
    }

    fn is_set(&self) -> bool {
        self.contains_wild() || self.is_non_wild_set()
    }

    fn contains_wild(&self) -> bool {
        for i in 0..3 {
            if let Card::Wild = self.cards[i] {
                return true;
            }
        }
        false
    }

    // returns whether the 3 cards contain no wilds but still form a set
    // (i.e. 3 of a kind or 1 of each kind)
    fn is_non_wild_set(&self) -> bool {
        match (self.cards[0], self.cards[1], self.cards[2]) {
            (Card::Territory(_, symbol0),
             Card::Territory(_, symbol1),
             Card::Territory(_, symbol2)) => {
                if symbol0 == symbol1 && symbol1 == symbol2 {
                    true
                } else if symbol0 != symbol1 && symbol1 != symbol2 && symbol0 != symbol2 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum CardSymbol {
    Infantry,
    Cavalry,
    Artillery,
}

#[derive(Copy, Clone)]
enum Card {
    Territory(TerritoryId, CardSymbol),
    Wild,
}

// TODO
type Reinforcement = ();
type Attack = ();
type Move = ();

trait Player {
    // called at the beginning of the turn, prompts the player to turn in a Risk
    // set.
    fn make_trade(&self, other_reinf: u8, necessary: bool) -> Option<Trade>;

    // called after a potential Risk set trade, prompts the player to distribute
    // available reinforcements
    fn distrib_reinforcements(&self, reinf: u8) -> Reinforcement;

    // called after reinforcements are distributed, prompts player to make an attack
    fn make_attack(&self) -> Attack;

    // called if an attack succeeds. prompts the player to move available armies
    // from the attacking territory to the newly occupied territory
    fn make_combat_move(&self) -> Move;

    // called exactly once per turn after all attacks are completed. prompts the user
    // to fortify a territory?
    fn fortify(&self) -> Move;

    fn add_cards(&mut self, cards: Vec<Card>);

    fn get_cards(&self) -> &[Card];

    fn remove_card(&mut self, usize);
}


#[derive(Clone)]
struct RandomPlayer {
    cards: Vec<Card>,
    // determines how often we trade in a set when it's not necessary
    // ('nnt' stands for non necessary trade-in
    param_nnt: f64,
}

impl RandomPlayer {
    // returns a vector of random players (each player is a trait object)
    pub fn make_random_players(number: usize) -> Vec<Box<Player>> {
        let mut players = vec![];
        for i in 0..number {
            let player = RandomPlayer {
                cards: vec![],
                param_nnt: rand::thread_rng().gen_range(0., 1.),
            };

            players.push(Box::new(player) as Box<Player>);
        }
        players
    }
}

impl Player for RandomPlayer {
    fn make_trade(&self, other_reinf: u8, necessary: bool) -> Option<Trade> {
        // if necessary or not necessary but a random roll exceeded k for some k in [0, 1]
        // then we make a trade. Identify all of the sets and pick one at
        // random.
        let x = rand::thread_rng().gen_range(0., 1.);
        if !necessary && x < self.param_nnt {
            return None;
        }

        // clone the card list and shuffle it
        let mut cards = self.cards.clone();
        let N = cards.len();

        rand::thread_rng().shuffle(&mut cards);

        // exhaustively search all subsets of order 3 to see if one is a set
        for i in 0..(N - 2) {
            for j in (i + 1)..(N - 1) {
                for k in (j + 1)..N {
                    let possible_trade = Trade::new([cards[i], cards[j], cards[k]]);
                    if possible_trade.is_set() {
                        return Some(possible_trade);
                    }
                }
            }
        }

        None
    }

    fn distrib_reinforcements(&self, reinf: u8) -> Reinforcement {
        unimplemented!()
    }

    fn make_attack(&self) -> Attack {
        unimplemented!()
    }

    fn make_combat_move(&self) -> Move {
        unimplemented!()
    }

    fn fortify(&self) -> Move {
        unimplemented!()
    }

    fn add_cards(&mut self, cards: Vec<Card>) {
        self.cards.extend(cards);
    }

    fn get_cards(&self) -> &[Card] {
        &self.cards[..]
    }

    fn remove_card(&mut self, i: usize) {
        self.cards.remove(i);
    }
}


struct GameManager {
    players: Vec<Box<Player>>,
    board: Box<GameBoard>,
    next_player: usize,
}

impl GameManager {
    pub fn new_game(players: Vec<Box<Player>>) -> GameManager {
        let num_players = players.len() as u8;
        let board = StandardGameBoard::new(num_players,
                                           distrib_board_territories_randomly(num_players));

        GameManager {
            players: players,
            board: Box::new(board),
            next_player: 0,
        }
    }

    fn next_player(&mut self) -> PlayerId {
        let next = self.next_player as PlayerId;
        self.next_player = (self.next_player + 1) % self.players.len();
        next
    }

    pub fn run(&mut self) {
        let mut current_player = self.next_player();

        while !self.board.game_is_over() {
            if !self.board.player_is_defeated(current_player) {
                self.process_trade(current_player);
            }
            current_player = self.next_player();
        }
    }

    pub fn process_trade(&self, current_id: PlayerId) {
        if self.board.get_num_cards(current_id) > 2 {
            let current_player = self.get_player(current_id);
            let trade_necessary = self.board.get_num_cards(current_id) > 4;
            let terr_reinf = self.board.get_territory_reinforcements(current_id);

            loop {
                let chosen_trade = current_player.make_trade(terr_reinf, trade_necessary);
                if self.verify_trade(chosen_trade, trade_necessary) {
                    if chosen_trade.is_some() {
                        // TODO: carry out trade
                    }
                    break;
                } else {
                    println!("Invalid trade chosen. Choose again.");
                }
            }
        }
    }

    fn verify_trade(&self, trade: Option<Trade>, necessary: bool) -> bool {
        match trade {
            None => !necessary,
            Some(trade) => trade.is_set(),
        }
    }

    fn get_player(&self, id: PlayerId) -> &Player {
        self.players[id as usize].as_ref()
    }
}

// distributes the territories as equally as possible among the available players
fn distrib_board_territories_randomly(num_players: u8) -> GameBoardTerritories {
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


fn main() {
    println!("Hello, world!");
    let mgr = GameManager::new_game(RandomPlayer::make_random_players(4));
}
