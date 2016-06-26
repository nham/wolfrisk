extern crate petgraph;
extern crate rand;

use rand::Rng;
use std::collections::{HashMap, HashSet};

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

type CardId = usize;

struct Trade {
    pub cards: [CardAndId; 3],
}

impl Trade {
    fn new(cards: [CardAndId; 3]) -> Trade {
        Trade { cards: cards }
    }

    fn cards_as_tuple(&self) -> (Card, Card, Card) {
        (self.cards[0].0, self.cards[1].0, self.cards[2].0)
    }

    fn is_set(&self) -> bool {
        self.contains_wild() || self.is_non_wild_set()
    }

    fn contains_wild(&self) -> bool {
        self.num_wild() > 0
    }

    fn num_wild(&self) -> usize {
        let mut count = 0;
        for i in 0..3 {
            if self.cards[i].0.is_wild() {
                count += 1;
            }
        }
        count
    }

    // returns whether the 3 cards contain no wilds but still form a set
    // (i.e. 3 of a kind or 1 of each kind)
    fn is_non_wild_set(&self) -> bool {
        match self.cards_as_tuple() {
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

    // TODO: this should probably be in a Rules object
    // or something
    fn value(&self) -> NumArmies {
        if !self.is_set() {
            0
        } else {
            match self.cards_as_tuple() {
                (Card::Territory(_, sym0),
                 Card::Territory(_, sym1),
                 Card::Territory(_, sym2)) => {
                    if sym0 == sym1 && sym1 == sym2 {
                        Trade::value_for_uniform_set(sym0)
                    } else if sym0 != sym1 && sym1 != sym2 && sym0 != sym2 {
                        10
                    } else {
                        0
                    }
                }
                cards => {
                    // trade contains a wild
                    if self.num_wild() == 2 {
                        10
                    } else {
                        let cards = vec![cards.0, cards.1, cards.2];

                        let i = if cards[0].is_wild() {
                            0
                        } else if cards[1].is_wild() {
                            1
                        } else {
                            2
                        };

                        if cards[(i + 1) % 3] == cards[(i + 2) % 3] {
                            let sym = cards[(i + 1) % 3].get_symbol()
                                                        .expect("There seems to be more than one wild, which .num_wild() did not detect.");
                            Trade::value_for_uniform_set(sym)
                        } else {
                            10
                        }
                    }
                },
            }
        }
    }


    // value for a set where all cards have the given CardSymbol
    fn value_for_uniform_set(x: CardSymbol) -> NumArmies {
        match x {
            CardSymbol::Infantry => 4,
            CardSymbol::Cavalry => 6,
            CardSymbol::Artillery => 8,
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

    fn get_symbol(&self) -> Option<CardSymbol> {
        match *self {
            Card::Territory(_, sym) => Some(sym),
            Card::Wild => None,
        }
    }

    fn get_territory(&self) -> Option<TerritoryId> {
        match *self {
            Card::Territory(terr, _) => Some(terr),
            Card::Wild => None,
        }
    }

}

type CardAndId = (Card, CardId);

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
    fn make_trade(&self, cards: &[CardAndId], other_reinf: NumArmies, necessary: bool) -> Option<Trade>;

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
}


struct RandomPlayer {
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
                param_nnt: rand::thread_rng().gen_range(0., 1.),
                param_attack: rand::thread_rng().gen_range(0., 1.),
            };

            players.push(Box::new(player) as Box<Player>);
        }
        players
    }
}

impl Player for RandomPlayer {
    fn make_trade(&self, cards: &[CardAndId], other_reinf: NumArmies, necessary: bool) -> Option<Trade> {
        // if necessary or not necessary but a random roll exceeded k for some k in [0, 1]
        // then we make a trade. Identify all of the sets and pick one at
        // random.

        let x = rand::thread_rng().gen_range(0., 1.);
        if !necessary && x < self.param_nnt {
            return None;
        }

        // clone the card list and shuffle it
        let mut card_idxs = vec![];
        let N = cards.len();
        println!("  N = {}", N);
        for i in 0..N {
            card_idxs.push(i);
        }
        rand::thread_rng().shuffle(&mut card_idxs);

        // exhaustively search all subsets of order 3 to see if one is a set
        for i in 0..(N - 2) {
            for j in (i + 1)..(N - 1) {
                for k in (j + 1)..N {
                    let cai_i = cards[card_idxs[i]];
                    let cai_j = cards[card_idxs[j]];
                    let cai_k = cards[card_idxs[k]];
                    let possible_trade = Trade::new([cai_i, cai_j, cai_k]);

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
}


struct CardManager {
    cards: Vec<Card>,
    available: HashSet<CardId>,
    discarded: HashSet<CardId>,
    player_cards: HashMap<PlayerId, HashSet<CardId>>,
}

impl CardManager {
    pub fn new(num_players: usize, cards: Vec<Card>) -> CardManager {
        let mut map = HashMap::new();
        for i in 0..(num_players as PlayerId) {
            map.insert(i, HashSet::new());
        }
        let N = cards.len();
        CardManager {
            cards: cards,
            available: (0..N).collect(),
            discarded: HashSet::new(),
            player_cards: map,
        }
    }

    pub fn standard_card_manager(num_players: usize) -> CardManager {
        let mut cards = Vec::new();
        let offset = rand::thread_rng().gen_range(0, 3);
        for i in 0..42 {
            cards.push(Card::Territory(i as TerritoryId,
                                       CardSymbol::from_usize((i + offset) % 3).unwrap()))
        }
        for _ in 0..2 {
            cards.push(Card::Wild);
        }
        CardManager::new(num_players, cards)
    }

    pub fn player_discard_card(&mut self, player: PlayerId, cid: CardId) {
        match self.player_cards.get_mut(&player) {
            None => panic!("Player {} is invalid", player),
            Some(player_cards) => {
                if player_cards.remove(&cid) {
                    self.discarded.insert(cid);
                }
            },
        }
    }

    pub fn get_available(&self) -> Vec<CardId> {
        self.available.iter().map(|&cid| cid).collect()
    }

    pub fn get_player_cards(&self, player: PlayerId) -> Vec<CardAndId> {
        match self.player_cards.get(&player) {
            None => panic!("Player {} is invalid", player),
            Some(cards) => cards.iter()
                                .map(|&id| (self.cards[id], id))
                                .collect(),
        }
    }

    pub fn get_num_player_cards(&self, player: PlayerId) -> usize {
        let cards = self.player_cards.get(&player);
        match self.player_cards.get(&player) {
            None => panic!("Player {} is invalid", player),
            Some(cards) => cards.len(),
        }
    }

    // when the `available` pile is empty, add in the discarded cards.
    fn recycle_discard_pile(&mut self) {
        self.available.extend(self.discarded.drain());
    }

    pub fn draw_random_for_player(&mut self, player: PlayerId) {
        if self.available.len() == 0 {
            self.recycle_discard_pile();
        }

        match self.player_cards.get_mut(&player) {
            None => panic!("Player {} is invalid", player),
            Some(cards) => {
                // clone the card list and shuffle it
                let mut cids: Vec<_> = self.available.iter().map(|&c| c).collect();
                rand::thread_rng().shuffle(&mut cids);
                cards.insert(cids[0]);
            },
        }
    }

    fn player_has_3_cards(&self, player: PlayerId, cards: [CardAndId; 3]) -> bool {
        match self.player_cards.get(&player) {
            None => false,
            Some(player_cards) => {
                for i in 0..3 {
                    // if cards[i].1 is not in player_cards, or if it is
                    // but self.cards[ cards[i].1 ] doesn't match cards[1].0,
                    // then return false
                    let cid = cards[i].1;
                    let matches: Vec<_> = player_cards.iter()
                                                      .filter(|&x| *x == cid)
                                                      .collect();
                    if matches.len() != 1 || cards[i].0 != self.cards[cid] {
                        return false;
                    }
                }
                true
            }
        }
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
    // their turn (also the discard pile is contained in this data structure)
    cards: CardManager,

    curr_player: usize,
}

impl GameManager {
    pub fn new_game(players: Vec<Box<Player>>) -> GameManager {
        let num_players = players.len();
        let board = StandardGameBoard::randomly_distributed(num_players as u8);

        GameManager {
            players: players,
            board: Box::new(board),
            cards: CardManager::standard_card_manager(num_players),
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
    fn process_trade(&mut self, player: PlayerId) -> NumArmies {
        if self.cards.get_num_player_cards(player) < 3 {
            return 0;
        }

        let trade_necessary = self.cards.get_num_player_cards(player) > 4;
        let terr_reinf = self.board.get_territory_reinforcements(player);
        let player_cards = self.cards.get_player_cards(player);

        let mut reinf = 0;
        loop {
            let chosen_trade = self.get_player(player)
                                   .make_trade(&player_cards[..], terr_reinf, trade_necessary);
            if self.verify_trade(player, &chosen_trade, trade_necessary) {
                match chosen_trade {
                    Some(trade) => {
                        println!("Player {} is trading in {:?}", player, trade.cards);
                        reinf += self.perform_trade(player, trade);
                    }
                    None => {
                        // assume that the player doesn't want to trade in anything else
                        break;
                    }
                }

                if self.cards.get_num_player_cards(player) < 3 {
                    break;
                }

            } else {
                println!("Invalid trade chosen. Choose again.");
            }
        }
        reinf
    }

    // returns the number of bonus armies granted by the trade-in
    fn perform_trade(&mut self, player: PlayerId, trade: Trade) -> NumArmies {
        for i in 0..3 {
            self.cards.player_discard_card(player, trade.cards[i].1);
            match trade.cards[i].0.get_territory() {
                None => continue,
                Some(tid) => {
                    if self.board.get_owner(tid) == player {
                        self.board.add_armies(tid, 2);
                    }
                },
            }
        }

        trade.value()
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
            self.cards.draw_random_for_player(curr_id);
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
            Some(ref trade) => self.cards.player_has_3_cards(player, trade.cards) &&
                               trade.is_set(),
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
}


fn main() {
    println!("Hello, world!");
    let mut mgr = GameManager::new_game(RandomPlayer::make_random_players(4));
    mgr.run();
}
