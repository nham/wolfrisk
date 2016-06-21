extern crate petgraph;
extern crate rand;

use rand::Rng;
use std::collections::HashMap;

use board::{GameBoard, GameMap, StandardGameBoard};

mod board;

pub const NUM_TERRITORIES: usize = 42;

pub type TerritoryId = u8;
pub type PlayerId = u8;
type NumArmies = u16;

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum CardSymbol {
    Infantry,
    Cavalry,
    Artillery,
}

impl CardSymbol {
    fn from_usize(x: usize) -> Option<CardSymbol> {
        match x {
            0 => Some(CardSymbol::Infantry),
            1 => Some(CardSymbol::Cavalry),
            2 => Some(CardSymbol::Artillery),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Card {
    Territory(TerritoryId, CardSymbol),
    Wild,
}


struct Reinforcement {
    player: PlayerId,
    reinf: HashMap<TerritoryId, NumArmies>,
}

impl Reinforcement {
    fn new(player: PlayerId, reinf: HashMap<TerritoryId, NumArmies>) -> Reinforcement {
        Reinforcement {
            player: player,
            reinf: reinf,
        }
    }

    fn iter(&self) -> std::collections::hash_map::Iter<TerritoryId, NumArmies> {
        self.reinf.iter()
    }

    fn player(&self) -> PlayerId {
        self.player
    }
}


struct AttackTerritoryInfo {
    pub id: TerritoryId,
    pub armies: NumArmies,
    pub adj_enemies: Vec<TerritoryId>,
}

struct Attack {
    pub attacker: PlayerId,
    pub origin: TerritoryId,
    pub target: TerritoryId,
    pub amount: AttackAmount,
}

impl Attack {
    fn new(attacker: PlayerId,
           origin: TerritoryId,
           target: TerritoryId,
           amount: AttackAmount)
           -> Attack {
        Attack {
            attacker: attacker,
            origin: origin,
            target: target,
            amount: amount,
        }
    }
}

#[derive(Copy, Clone)]
enum AttackAmount {
    One = 1,
    Two = 2,
    Three = 3,
}

impl AttackAmount {
    // assumes n != 0
    fn max_from_num_armies(n: NumArmies) -> AttackAmount {
        if n >= 3 {
            AttackAmount::Three
        } else if n == 2 {
            AttackAmount::Two
        } else {
            AttackAmount::One
        }
    }
}

// TODO
type Move = ();

trait Player {
    // called at the beginning of the turn, prompts the player to turn in a set
    fn make_trade(&self, other_reinf: NumArmies, necessary: bool) -> Option<Trade>;

    // called after a potential set trade, prompts the player to distribute
    // available reinforcements
    fn distrib_reinforcements(&self, PlayerId, NumArmies, &[TerritoryId]) -> Reinforcement;

    // called after reinforcements are distributed, prompts player to make an attack
    // takes a slice where each element is an information data structure corresponding
    // to one of the territories that the player owns.
    fn make_attack(&self, PlayerId, &[AttackTerritoryInfo]) -> Option<Attack>;

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
    // determines how often the player trades in a set when it's not necessary
    // ('nnt' stands for non necessary trade-in
    param_nnt: f64,

    // determines how often the player attacks from a territory capable of
    // attacking
    param_attack: f64,
}

impl RandomPlayer {
    // returns a vector of random players (each player is a trait object)
    pub fn make_random_players(number: usize) -> Vec<Box<Player>> {
        let mut players = vec![];
        for _ in 0..number {
            let player = RandomPlayer {
                cards: vec![],
                param_nnt: rand::thread_rng().gen_range(0., 1.),
                param_attack: rand::thread_rng().gen_range(0., 1.),
            };

            players.push(Box::new(player) as Box<Player>);
        }
        players
    }
}

impl Player for RandomPlayer {
    fn make_trade(&self, other_reinf: NumArmies, necessary: bool) -> Option<Trade> {
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

    fn distrib_reinforcements(&self,
                              player: PlayerId,
                              reinf: NumArmies,
                              owned: &[TerritoryId])
                              -> Reinforcement {
        let mut terr_reinf = HashMap::new();
        for i in 0..reinf {
            // pick a random owned territory to assign this reinforcement to
            let rand_idx = rand::thread_rng().gen_range(0, owned.len());
            let rand_terr = owned[rand_idx];
            let amt = terr_reinf.entry(rand_terr).or_insert(0);
            *amt += 1;
        }

        Reinforcement::new(player, terr_reinf)
    }

    fn make_attack(&self, player: PlayerId, terr_info: &[AttackTerritoryInfo]) -> Option<Attack> {
        for info in terr_info.iter() {
            if info.armies > 1 && info.adj_enemies.len() > 0 {
                let x = rand::thread_rng().gen_range(0., 1.);
                if x >= self.param_attack {
                    let rand_idx = rand::thread_rng().gen_range(0, info.adj_enemies.len());
                    let defender = info.adj_enemies[rand_idx];
                    return Some(Attack::new(player,
                                            info.id,
                                            defender,
                                            AttackAmount::max_from_num_armies(info.armies - 1)));
                }
            }
        }
        None
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


// should this be a trait instead or is that overkill?
struct Deck {
    available: Vec<Card>,
    discarded: Vec<Card>,
}

impl Deck {
    pub fn new(cards: Vec<Card>) -> Deck {
        Deck {
            available: cards,
            discarded: Vec::new(),
        }
    }

    pub fn standard_deck() -> Deck {
        let mut cards = Vec::new();
        let offset = rand::thread_rng().gen_range(0, 3);
        for i in 0..42 {
            cards.push(Card::Territory(i as TerritoryId,
                                       CardSymbol::from_usize((i + offset) % 3).unwrap()))
        }
        for _ in 0..2 {
            cards.push(Card::Wild);
        }
        Deck::new(cards)
    }

    pub fn discard(&mut self, card: Card) {
        unimplemented!()
    }

    pub fn draw_random(&mut self) -> Card {
        unimplemented!()
    }

    pub fn get_available(&self) -> &[Card] {
        &self.available[..]
    }
}


struct GameManager {
    players: Vec<Box<Player>>,
    board: Box<GameBoard>,

    // the cards available to be given to a player who conquers a territory in
    // their turn
    cards: Deck,

    curr_player: usize,
}

impl GameManager {
    pub fn new_game(players: Vec<Box<Player>>) -> GameManager {
        let num_players = players.len() as u8;
        let board = StandardGameBoard::randomly_distributed(num_players);

        GameManager {
            players: players,
            board: Box::new(board),
            cards: Deck::standard_deck(),
            curr_player: 0,
        }
    }

    fn current_player(&self) -> PlayerId {
        self.curr_player as PlayerId
    }

    // start the next player's turn, return that player's ID
    fn next_player(&mut self) -> PlayerId {
        self.curr_player = (self.curr_player + 1) % self.players.len();
        self.curr_player as PlayerId
    }

    pub fn run(&mut self) {
        self.log_starting_game();
        let mut current_player = self.current_player();

        const MAX_NUM_TURNS: usize = 100;
        let mut turn = 0;

        while !self.board.game_is_over() {
            if !self.board.player_is_defeated(current_player) {
                turn += 1;
                let trade_reinf = self.process_trade(current_player);
                self.process_reinforcement(current_player, trade_reinf);
                self.process_attack(current_player);

                if turn >= MAX_NUM_TURNS {
                    println!("MAX_NUM_TURNS exceeded, terminating game");
                    break;
                }
            }
            current_player = self.next_player();
        }
    }

    pub fn log_starting_game(&self) {
        println!("Starting a game with {} players.", self.players.len());
        println!("Deck:");
        for card in self.cards.get_available() {
            println!("Card: {:?}", card);
        }
    }

    // returns the number of extra reinforcements resulting from trading cards in
    pub fn process_trade(&self, current_id: PlayerId) -> NumArmies {
        if self.board.get_num_cards(current_id) > 2 {
            let current_player = self.get_player(current_id);
            let trade_necessary = self.board.get_num_cards(current_id) > 4;
            let terr_reinf = self.board.get_territory_reinforcements(current_id);

            let mut reinf = 0;
            loop {
                let chosen_trade = current_player.make_trade(terr_reinf, trade_necessary);
                if self.verify_trade(chosen_trade, trade_necessary) {
                    if chosen_trade.is_some() {
                        // TODO: carry out trade
                        // reinf += reinforcements from carrying out the trade-in
                    }

                    if self.board.get_num_cards(current_id) < 5 {
                        break;
                    }
                } else {
                    println!("Invalid trade chosen. Choose again.");
                }
            }
            return reinf;
        }
        0
    }

    pub fn process_reinforcement(&mut self, curr_id: PlayerId, trade_reinf: NumArmies) {
        let owned = self.board.get_owned_territories(curr_id);

        // calculate reinf
        let reinf_amt = self.board.get_territory_reinforcements(curr_id) + trade_reinf;

        println!("\nPlayer {} is distributing {} reinforcements", curr_id, reinf_amt);
        println!("==========");

        loop {
            let chosen_reinf = self.get_player(curr_id)
                                   .distrib_reinforcements(curr_id, reinf_amt, &owned[..]);
            if self.verify_reinf(reinf_amt, &chosen_reinf) {
                for (&terr, &reinf) in chosen_reinf.iter() {
                    if reinf > 0 {
                        let owner = self.board.get_owner(terr);
                        let num_armies = self.board.get_num_armies(terr);
                        self.board.set_territory(terr, owner, num_armies + reinf);
                        println!("  territory {} gained {} units (now {} in total)",
                                 terr,
                                 reinf,
                                 self.board.get_num_armies(terr));
                    }
                }
                break;
            } else {
                println!("Invalid reinforcement chosen. Choose again.");
            }
        }
    }

    fn generate_adj_enemy_info(&self,
                               curr_id: PlayerId,
                               owned: &[TerritoryId])
                               -> Vec<AttackTerritoryInfo> {
        let mut attack_info = Vec::new();
        for &terr in owned.iter() {
            // for each territory get the list of adjacent enemy territories
            attack_info.push(AttackTerritoryInfo {
                id: terr,
                armies: self.board.get_num_armies(terr),
                adj_enemies: self.board.game_map()
                                       .get_neighbors(terr)
                                       .into_iter()
                                       .filter(|&tid| self.board.is_enemy_territory(curr_id, tid))
                                       .collect(),
            });
        }
        attack_info
    }

    pub fn process_attack(&self, curr_id: PlayerId) {
        // prompt the player for a sequence of attacks:
        let owned = self.board.get_owned_territories(curr_id);
        let attack_info = self.generate_adj_enemy_info(curr_id, &owned[..]);

        let mut conquered_one = false;

        loop {
            let chosen_attack = self.get_player(curr_id)
                                    .make_attack(curr_id, &attack_info[..]);
            match chosen_attack {
                None => break,
                Some(attack) => {
                    if self.verify_attack(&attack) {
                        // TODO: perform the attack
                        // if all enemies were eliminated, conquered_one = true
                        unimplemented!()
                    } else {
                        println!("Attack chosen is invalid. Choose again");
                    }
                }
            }

        }

        if conquered_one {
            // TODO: give a random card to the player
            unimplemented!()
        }
    }

    fn verify_trade(&self, trade: Option<Trade>, necessary: bool) -> bool {
        // TODO: verify that player owns each card it's trying to trade in?
        match trade {
            None => !necessary,
            Some(trade) => trade.is_set(),
        }
    }

    fn verify_reinf(&self, reinf_amt: NumArmies, reinf: &Reinforcement) -> bool {
        let mut total_amt = 0;
        for (&terr, &amt) in reinf.iter() {
            total_amt += amt;
            if self.board.is_enemy_territory(reinf.player(), terr) {
                return false;
            }
        }
        total_amt == reinf_amt
    }

    fn verify_attack(&self, attack: &Attack) -> bool {
        // if there are that many excess units on the origin territory
        // and the target territory is actually an adjacent enemy
        // then the attack is valid. otherwise, not.
        let can_attack_with = self.board.get_num_armies(attack.origin) - 1;
        self.board.get_owner(attack.origin) == attack.attacker
            && can_attack_with >= (attack.amount as NumArmies)
            && self.board.game_map().are_adjacent(attack.origin, attack.target)
            && self.board.is_enemy_territory(attack.attacker, attack.target)
    }

    fn get_player(&self, id: PlayerId) -> &Player {
        self.players[id as usize].as_ref()
    }
}


fn main() {
    println!("Hello, world!");
    let mut mgr = GameManager::new_game(RandomPlayer::make_random_players(4));
    mgr.run();
}
