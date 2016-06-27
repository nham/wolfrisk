pub use self::random_player::RandomPlayer;
use super::{PlayerId, TerritoryId, NumArmies, CardAndId, AttackTerritories};
use super::{GameBoard, Trade, Reinforcement, Attack, Move};

mod random_player;

pub trait Player {
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

    // called once per turn after all attacks are completed. prompts the user to
    // fortify a territory
    fn fortify(&self, PlayerId, &GameBoard) -> Option<Move>;
}

