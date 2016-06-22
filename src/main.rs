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


fn attacking_allowed(pool: NumArmies) -> NumArmies {
    max_allowed(3, pool)
}

fn defending_allowed(pool: NumArmies) -> NumArmies {
    max_allowed(2, pool)
}

// given `max` and `pool`, returns min(`max`, `pool`)
fn max_allowed(max: NumArmies, pool: NumArmies) -> NumArmies {
    if pool > max {
        max
    } else {
        pool
    }
}

#[derive(Copy, Clone)]
struct Trade {
    pub cards: [Card; 3],
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
            if self.cards[i].0.is_wild() {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Card {
    Territory(TerritoryId, CardSymbol),
    Wild,
}

impl Card {
    fn is_wild(&self) -> bool {
        if let &Card::Wild = self {
            true
        } else {
            false
        }
    }
}


struct Reinforcement {
    reinf: HashMap<TerritoryId, NumArmies>,
}

impl Reinforcement {
    fn new(reinf: HashMap<TerritoryId, NumArmies>) -> Reinforcement {
        Reinforcement { reinf: reinf }
    }

    fn iter(&self) -> std::collections::hash_map::Iter<TerritoryId, NumArmies> {
        self.reinf.iter()
    }
}


struct AttackTerritoryInfo {
    pub id: TerritoryId,
    pub armies: NumArmies,
    pub adj_enemies: Vec<TerritoryId>,
}

type AttackTerritories = HashMap<TerritoryId, AttackTerritoryInfo>;

struct Attack {
    pub origin: TerritoryId,
    pub target: TerritoryId,
    pub amount_attacking: NumArmies,
}

impl Attack {
    fn new(origin: TerritoryId,
           target: TerritoryId,
           amount_attacking: NumArmies)
           -> Attack {
        Attack {
            origin: origin,
            target: target,
            amount_attacking: amount_attacking,
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
    fn distrib_reinforcements(&self, NumArmies, &[TerritoryId]) -> Reinforcement;

    // called after reinforcements are distributed, prompts player to make an attack
    // takes a slice where each element is an information data structure corresponding
    // to one of the territories that the player owns.
    fn make_attack(&self, &AttackTerritories) -> Option<Attack>;

    // called if an attack succeeds. prompts the player to move available armies
    // from the attacking territory to the newly occupied territory
    fn make_combat_move(&self) -> Move;

    // called exactly once per turn after all attacks are completed. prompts the user
    // to fortify a territory?
    fn fortify(&self) -> Move;

    fn add_cards(&mut self, cards: Vec<Card>);

    fn get_cards(&self) -> &[Card];

    fn remove_card(&mut self, usize);

    fn add_card(&mut self, card: Card) {
        self.add_cards(vec![card]);
    }

    fn has_3_cards(&self, cards: [Card; 3]) -> bool {
        let mut has: [bool; 3] = [false, false, false];
        for &card in self.get_cards().iter() {
            for i in 0..3 {
                if !has[i] && card == cards[i] {
                    has[i] = true;
                }
            }
        }

        has[0] && has[1] && has[2]
    }

    fn get_num_cards(&self) -> usize {
        self.get_cards().len()
    }
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

        Reinforcement::new(terr_reinf)
    }

    fn make_attack(&self, terr_info: &AttackTerritories) -> Option<Attack> {
        for info in terr_info.values() {
            if info.armies > 1 && info.adj_enemies.len() > 0 {
                let x = rand::thread_rng().gen_range(0., 1.);
                if x >= self.param_attack {
                    let rand_idx = rand::thread_rng().gen_range(0, info.adj_enemies.len());
                    let defender = info.adj_enemies[rand_idx];
                    // TODO: this always takes the max amount that can be attacked with
                    // should it be something different?
                    return Some(Attack::new(info.id,
                                            defender,
                                            attacking_allowed(info.armies - 1)));
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
        self.discarded.push(card);
    }

    pub fn draw_random(&mut self) -> Card {
        let i = rand::thread_rng().gen_range(0, self.available.len());
        self.available.remove(i)
    }

    pub fn get_available(&self) -> &[Card] {
        &self.available[..]
    }
}


// odds from https://www.kent.ac.uk/smsas/personal/odl/riskfaq.htm#3.2
fn one_rolled_1(attacker: NumArmies, defender: NumArmies) -> Option<[f64; 2]> {
    match (attacker, defender) {
        (1, 1) => Some([0.5833, 0.4167]),
        (2, 1) => Some([0.4213, 0.5787]),
        (3, 1) => Some([0.3403, 0.6597]),
        (1, 2) => Some([0.7454, 0.2546]),
        _ => None,
    }
}

// odds from https://www.kent.ac.uk/smsas/personal/odl/riskfaq.htm#3.2
fn both_rolled_at_least_2(attacker: NumArmies, defender: NumArmies) -> Option<[f64; 3]> {
    match (attacker, defender) {
        (2, 2) => Some([0.4483, 0.2276, 0.3241]),
        (3, 2) => Some([0.2926, 0.3717, 0.3358]),
        _ => None,
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

        const MAX_NUM_TURNS: usize = 20;
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
    // isn't this wrong? aren't there two different behaviors? if you are at the beginning
    // of a turn, you can turn in as many as you want
    // but during an attack you must turn in only until you have > 5, then you have to stop
    fn process_trade(&mut self, current_id: PlayerId) -> NumArmies {
        if self.get_player(current_id).get_num_cards() < 3 {
            return 0;
        }

        let trade_necessary = self.get_player(current_id).get_num_cards() > 4;
        let terr_reinf = self.board.get_territory_reinforcements(current_id);

        let mut reinf = 0;
        loop {
            // TODO: maybe the player should just give indices
            // and the GameManager will remove them somehow?
            // I think the key here might be moving the cards out of
            // the Player datastructure
            let chosen_trade = self.get_player(current_id)
                                   .make_trade(terr_reinf, trade_necessary);
            if self.verify_trade(current_id, &chosen_trade, trade_necessary) {
                match chosen_trade {
                    Some(trade) => {
                        println!("Player {} is trading in {:?}", current_id, trade.cards);
                        reinf += self.perform_trade(trade);
                    }
                    None => {}
                }

                if self.get_player(current_id).get_num_cards() < 3 {
                    break;
                }

            } else {
                println!("Invalid trade chosen. Choose again.");
            }
        }
        reinf
    }

    // returns the number of bonus armies granted by the trade-in
    fn perform_trade(&mut self, trade: Trade) -> NumArmies {
        unimplemented!()
    }

    pub fn process_reinforcement(&mut self, curr_id: PlayerId, trade_reinf: NumArmies) {
        let owned = self.board.get_owned_territories(curr_id);

        // calculate reinf
        let reinf_amt = self.board.get_territory_reinforcements(curr_id) + trade_reinf;

        println!("\nPlayer {} is distributing {} reinforcements",
                 curr_id,
                 reinf_amt);
        println!("==========");

        loop {
            let chosen_reinf = self.get_player(curr_id)
                                   .distrib_reinforcements(reinf_amt, &owned[..]);
            if self.verify_reinf(curr_id, reinf_amt, &chosen_reinf) {
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
                adj_enemies: self.board
                    .game_map()
                    .get_neighbors(terr)
                    .into_iter()
                    .filter(|&tid| self.board.is_enemy_territory(curr_id, tid))
                    .collect(),
            });
        }
        attack_info
    }

    pub fn process_attack(&mut self, curr_id: PlayerId) {
        // prompt the player for a sequence of attacks:
        let owned = self.board.get_owned_territories(curr_id);
        let mut attack_info = HashMap::new();
        for info in self.generate_adj_enemy_info(curr_id, &owned[..]).into_iter() {
            attack_info.insert(info.id, info);
        }

        let mut conquered_one = false;

        loop {
            let chosen_attack = self.get_player(curr_id).make_attack(&attack_info);
            match chosen_attack {
                None => break,
                Some(attack) => {
                    println!("Player {} is attacking with {} from {} to {}",
                             curr_id,
                             attack.amount_attacking,
                             attack.origin,
                             attack.target);
                    if self.verify_battle(curr_id, &attack) {
                        let conquered = self.perform_battle(&attack);

                        // update attack_info
                        if self.board.get_num_armies(attack.origin) == 1 {
                            attack_info.remove(&attack.origin);
                        } else {
                            let origin = attack_info.get_mut(&attack.origin).unwrap();
                            origin.armies = self.board.get_num_armies(attack.origin);
                        }

                        if conquered {
                            conquered_one = true;
                            // update attack_info
                            for (_, info) in attack_info.iter_mut() {
                                let mut conquered_ind = None;
                                for i in 0..info.adj_enemies.len() {
                                    if info.adj_enemies[i] == attack.target {
                                        conquered_ind = Some(i);
                                    }
                                }
                                if let Some(ind) = conquered_ind {
                                    info.adj_enemies.remove(ind);
                                }
                            }
                        }
                    } else {
                        println!("Attack chosen is invalid. Choose again");
                    }
                }
            }

        }

        if conquered_one {
            let random_card = self.cards.draw_random();
            self.give_card_to_player(curr_id, random_card);
        }
    }

    // this function is called once the proposed attack has been verified
    // to be a valid attack
    // Returns true if the battle resulted in the defending territory being
    // conquered
    fn perform_battle(&mut self, attack: &Attack) -> bool {
        let num_enemy_armies = self.board.get_num_armies(attack.target);
        let amount_defending = defending_allowed(num_enemy_armies);
        let amount_attacking = attack.amount_attacking;

        // we are not rolling any dice here, we are just going to use
        // a uniform randomly variable and the probability tables
        let roll = rand::thread_rng().gen_range(0., 1.);

        let outcome: (NumArmies, NumArmies) =
            if amount_defending == 1 || amount_attacking == 1 {
                let dist = one_rolled_1(amount_attacking, amount_defending).unwrap();
                if roll <= dist[0] {
                    // attacker loses 1
                    (1, 0)
                } else {
                    // defender loses 1
                    (0, 1)
                }
            } else {
                let dist = both_rolled_at_least_2(amount_attacking, amount_defending).unwrap();
                if roll <= dist[0] {
                    // attacker loses 2
                    (2, 0)
                } else if roll > dist[0] && roll <= (dist[0] + dist[1]) {
                    // defender loses 2
                    (0, 2)
                } else {
                    // both lose 1
                    (1, 1)
                }
            };

        let must_commit = amount_attacking - outcome.0;

        if outcome.0 > 0 {
            self.board.remove_armies(attack.origin, outcome.0);
            println!("Attacking territory {} lost {} units in battle",
                     attack.origin,
                     outcome.0);
        }

        if outcome.1 > 0 {
            self.board.remove_armies(attack.target, outcome.1);
            println!("Defending territory {} lost {} units in battle",
                     attack.target,
                     outcome.1);
        }

        if self.board.get_num_armies(attack.target) == 0 {
            self.board.remove_armies(attack.origin, must_commit);
            self.board.add_armies(attack.target, must_commit);
            println!("Territory {} was conquered, moving {} units over from {}",
                     attack.target,
                     must_commit,
                     attack.origin);
            true
            // TODO: prompt user for combat move
        } else {
            false
        }
    }

    fn verify_trade(&self, player: PlayerId, trade: &Option<Trade>, necessary: bool) -> bool {
        match *trade {
            None => !necessary,
            Some(ref trade) => self.get_player(player).has_3_cards(trade.cards) && trade.is_set(),
        }
    }

    fn verify_reinf(&self, player: PlayerId, reinf_amt: NumArmies, reinf: &Reinforcement) -> bool {
        let mut total_amt = 0;
        for (&terr, &amt) in reinf.iter() {
            total_amt += amt;
            if self.board.is_enemy_territory(player, terr) {
                return false;
            }
        }
        total_amt == reinf_amt
    }

    fn verify_battle(&self, player: PlayerId, attack: &Attack) -> bool {
        // if there are that many excess units on the origin territory
        // and the target territory is actually an adjacent enemy
        // then the attack is valid. otherwise, not.
        let can_attack_with = self.board.get_num_armies(attack.origin) - 1;
        self.board.get_owner(attack.origin) == player &&
        can_attack_with >= attack.amount_attacking &&
        self.board.game_map().are_adjacent(attack.origin, attack.target) &&
        self.board.is_enemy_territory(player, attack.target)
    }

    fn get_player(&self, id: PlayerId) -> &Player {
        self.players[id as usize].as_ref()
    }

    fn give_card_to_player(&mut self, id: PlayerId, card: Card) {
        println!("Player {} received card {:?}", id, card);
        self.players[id as usize].add_card(card);
    }
}


fn main() {
    println!("Hello, world!");
    let mut mgr = GameManager::new_game(RandomPlayer::make_random_players(4));
    mgr.run();
}
