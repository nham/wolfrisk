use rand::{self, Rng};
use std::collections::HashMap;
use super::Player;
use ::{PlayerId, TerritoryId, NumArmies, CardAndId, AttackTerritories};
use ::{GameBoard, GameMap, Trade, Reinforcement, Attack, Move};
use ::attacking_allowed;

pub struct RandomPlayer {
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
                    let defender = {
                        let mut adj_enemies: Vec<_> = info.adj_enemies.iter()
                                                                      .map(|&e| e)
                                                                      .collect();
                        rand::thread_rng().shuffle(&mut adj_enemies);
                        adj_enemies[0]
                    };

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

    fn fortify(&self, player: PlayerId, board: &GameBoard) -> Option<Move> {
        // generate a vector of (tid, list of owned territories adjacent to tid) items,
        // one for each territory owned by the player
        let mut terrs_w_adj_owned: Vec<_> = board.get_owned_territories(player)
                                               .into_iter()
                                               .map(|tid| {
            let adj_owned: Vec<_> = board.get_owned_territories(player)
                                         .into_iter()
                                         .filter(|&tid2| board.game_map().are_adjacent(tid, tid2)).collect();
            (tid, adj_owned)
           }).collect();

        // filter out territories with friendly neighbors
        terrs_w_adj_owned = terrs_w_adj_owned.into_iter()
                                             .filter(|&(_, ref owned)| owned.len() > 0)
                                             .collect();

        // filter out territories with only 1 army
        terrs_w_adj_owned = terrs_w_adj_owned.into_iter()
                                             .filter(|&(tid, _)| board.get_num_armies(tid) > 1)
                                             .collect();

        if terrs_w_adj_owned.len() == 0 {
            return None;
        }

        // pick a random owned territory that has at least one adjacent owned
        // territory.
        rand::thread_rng().shuffle(&mut terrs_w_adj_owned);
        let mut origin = &mut terrs_w_adj_owned[0];

        // pick a random destination territory
        rand::thread_rng().shuffle(&mut origin.1);
        let destination = origin.1[0];


        // pick a random int between 0 and get_num_armies(origin territory) - 1
        let rand_num_armies = rand::thread_rng().gen_range(0, board.get_num_armies(origin.0) - 1);
        Some(Move {
            origin: origin.0,
            destination: destination,
            amount: rand_num_armies,
        })

    }
}
