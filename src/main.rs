extern crate rand;

use rand::Rng;

const NUM_TERRITORIES: usize = 42;

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
    fn set_territory(&mut self, TerritoryId, PlayerId, u8);
    fn set_num_cards(&mut self, PlayerId, u8);
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
}

// TODO
type Trade = ();
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
}


#[derive(Clone)]
struct RandomPlayer;

impl RandomPlayer {
    // returns a vector of random players, each of them boxed.
    pub fn make_random_players(number: usize) -> Vec<Box<Player>> {
        let mut players = vec![];
        for i in 0..number {
            players.push(Box::new(RandomPlayer) as Box<Player>);
        }
        players
    }
}

impl Player for RandomPlayer {
    fn make_trade(&self, other_reinf: u8, necessary: bool) -> Option<Trade> {
        unimplemented!()
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
}


struct GameManager {
    players: Vec<Box<Player>>,
    board: Box<GameBoard>,
}

impl GameManager {
    pub fn new_game(players: Vec<Box<Player>>) -> GameManager {
        let num_players = players.len() as u8;
        let board = StandardGameBoard::new(
            num_players,
            distrib_board_territories_randomly(num_players)
        );

        GameManager {
            players: players,
            board: Box::new(board),
        }
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
