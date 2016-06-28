extern crate petgraph;
extern crate rand;

use std::collections::{HashMap, HashSet};

pub use board::{GameBoard, GameMap};
use player::{RandomPlayer, HumanPlayer};
use game_manager::GameManager;

mod board;
mod game_manager;
mod player;

pub const NUM_TERRITORIES: usize = 42;

pub type TerritoryId = u8;
pub type PlayerId = u8;
pub type NumArmies = u16;
pub type CardId = usize;
pub type CardAndId = (Card, CardId);
pub type AttackTerritories = HashMap<TerritoryId, AttackTerritoryInfo>;


pub struct Trade {
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


pub struct Reinforcement {
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


pub struct Attack {
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


pub struct Move {
    pub origin: TerritoryId,
    pub destination: TerritoryId,
    pub amount: NumArmies,
}


pub struct AttackTerritoryInfo {
    pub id: TerritoryId,
    pub armies: NumArmies,
    pub adj_enemies: HashSet<TerritoryId>,
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CardSymbol {
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
pub enum Card {
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

fn main() {
    println!("Hello, world!");
    let mut players = RandomPlayer::make_random_players(3);
    players.push(Box::new(HumanPlayer));
    let mut mgr = GameManager::new_game(players);
    mgr.run();
}
