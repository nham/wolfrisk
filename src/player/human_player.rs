use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::str::FromStr;

use super::Player;
use ::{PlayerId, TerritoryId, NumArmies, CardAndId, AttackTerritories};
use ::{GameBoard, Trade, Reinforcement, Attack, Move};

pub struct HumanPlayer;

// helper function for <HumanPlayer as Player>::make_trade
fn prompt_for_trade_cards() -> [usize; 3] {
    let mut idxs = [0, 0, 0];
    for i in 0..3 {
        loop {
            match prompt_and_parse::<usize>("Enter index of card to trade: ") {
                Err(_) => continue,
                Ok(index) => {
                    idxs[i] = index;
                }
            }
        }
    }
    idxs
}

impl Player for HumanPlayer {
    fn make_trade(&self, cards: &[CardAndId], other_reinf: NumArmies, necessary: bool) -> Option<Trade> {
        println!("Cards:");
        for card in cards.iter() {
            println!("{:?}", card);
        }

        println!("Reinforcement from territory: {}", other_reinf);
        println!("Necessary: {}", necessary);

        // prompt user to enter indices via command line
        loop {
            let input = prompt("Make trade? (y/n):").trim().to_ascii_lowercase();
            if input.len() == 1 {
                let chars: Vec<_> = input.chars().collect();
                match chars[0] {
                    'n' => return None,
                    'y' => {
                        let idxs = prompt_for_trade_cards();
                        return Some(Trade::new([cards[idxs[0]],
                                                cards[idxs[1]],
                                                cards[idxs[2]]]))
                    }
                    _ => continue,
                }
            }
        }
    }

    fn distrib_reinforcements(&self,
                              reinf_amt: NumArmies,
                              owned: &[TerritoryId])
                              -> Reinforcement {
        println!("Reinforcements to distribute: {}", reinf_amt);

        println!("Owned territories:");
        for terr in owned.iter() {
            print!("{:?} ", terr);
        }
        println!("");
        flush_stdout();

        let mut reinf = HashMap::new();

        let mut reinf_avail = reinf_amt;
        loop {
            if reinf_avail == 0 { break; }

            match prompt_and_parse::<TerritoryId>("Territory to reinforce: ") {
                Err(_) => continue,
                Ok(terr) => {
                    loop {
                        match prompt_and_parse::<NumArmies>("Number of armies to reinforce: ") {
                            Err(_) => continue,
                            Ok(num_armies) => {
                                if num_armies > reinf_avail {
                                    println!("Only {} available, can't reinforce that many.",
                                             reinf_avail);
                                    continue;
                                }

                                reinf_avail -= num_armies;

                                if let Some(old_num_armies) = reinf.get(&terr).map(|&x| x) {
                                    reinf.insert(terr, num_armies + old_num_armies);
                                } else {
                                    reinf.insert(terr, num_armies);
                                }

                                break;
                            },
                        }
                    }
                },
            }
        }

        Reinforcement::new(reinf)
    }

    fn make_attack(&self, terr_info: &AttackTerritories) -> Option<Attack> {
        // print out info
        for info in terr_info.values() {
            println!("Territory {} has {} units and adjacent enemies {:?}",
                     info.id, info.armies, info.adj_enemies.iter()
                                                           .map(|&x| x)
                                                           .collect::<Vec<_>>());
        }

        // ask if user wants to make an attack
        // if so, take in the origin and target territories and the number of
        // units to attack with
        loop {
            let input = prompt("Attack? (y/n):").trim().to_ascii_lowercase();
            if input.len() == 1 {
                let chars: Vec<_> = input.chars().collect();
                match chars[0] {
                    'n' => return None,
                    'y' => {
                        let origin = repeatedly_prompt_and_parse::<TerritoryId>(" Origin territory: ");
                        let target = repeatedly_prompt_and_parse::<TerritoryId>(" Target territory: ");
                        // TODO: this would be friendlier if checked whether the amount was within
                        // the available amount of units that can be attacked with
                        let num_armies = repeatedly_prompt_and_parse::<NumArmies>(" Number of armies: ");
                        return Some(Attack::new(origin,
                                                target,
                                                num_armies));
                    }
                    _ => continue,
                }
            }
        }
    }


    fn make_combat_move(&self) -> Move {
        unimplemented!()
    }

    fn fortify(&self, player: PlayerId, board: &GameBoard) -> Option<Move> {
        loop {
            let input = prompt("Fortify? (y/n):").trim().to_ascii_lowercase();
            if input.len() == 1 {
                let chars: Vec<_> = input.chars().collect();
                match chars[0] {
                    'n' => return None,
                    'y' => {
                        let origin = repeatedly_prompt_and_parse::<TerritoryId>(" Origin territory: ");
                        let dest = repeatedly_prompt_and_parse::<TerritoryId>(" Destination territory: ");
                        // TODO: this would be friendlier if checked whether the amount was within
                        // the available amount of units that can be moved
                        let num_armies = repeatedly_prompt_and_parse::<NumArmies>(" Number of armies to move: ");
                        return Some(Move {
                            origin: origin,
                            destination: dest,
                            amount: num_armies,
                        });
                    }
                    _ => continue,
                }
            }
        }
    }
    
}

// panics if it couldn't flush it
fn flush_stdout() {
    io::stdout().flush().expect("Couldn't flush stdout");
}

// keep prompting and parsing until one of the parses succeeds
fn repeatedly_prompt_and_parse<T: FromStr>(msg: &str) -> T {
    loop {
        match prompt_and_parse::<T>(msg) {
            Err(_) => continue,
            Ok(tid) => return tid,
        };
    }
}


fn prompt_and_parse<T: FromStr>(msg: &str) -> Result<T, <T as FromStr>::Err> {
    let mut input = prompt(msg);
    input.trim().parse::<T>()
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().expect("Couldn't flush stdout");
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_line(&mut buf).unwrap();
    buf
}
