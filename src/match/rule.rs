use super::{
    events::PlayerDied,
    on_match_update,
    player::{PlayerConfig, PlayerDamage},
    MatchConfig, MatchResult, MatchState,
};
use crate::player::Player;
use bevy::{app::AppExit, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum MatchWinner {
    /// No winner has been decided yet.
    Undecided,
    /// Match can no longer go on, but there is no apparent winner.
    NoContest,
    /// A singular player has won.
    Player(Player),
    /// A team of players has won.
    Team,
}

impl Default for MatchWinner {
    fn default() -> Self {
        Self::Undecided
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum MatchRule {
    Score,
    Stamina(f32),
    Stock(u8),
}

impl Default for MatchRule {
    fn default() -> Self {
        Self::Score
    }
}

impl MatchRule {
    /// Creates a PlayerDamage from a given rule and PlayerConfig.
    pub(super) fn create_damage(&self, config: &PlayerConfig) -> PlayerDamage {
        match self {
            Self::Score => PlayerDamage::Score {
                score: 0,
                damage: config.default_damage,
                default_damage: config.default_damage,
            },
            Self::Stock(stocks) => PlayerDamage::Stock {
                stocks: *stocks,
                damage: config.default_damage,
                default_damage: config.default_damage,
            },
            Self::Stamina(health) => PlayerDamage::Stamina {
                health: *health,
                full_health: *health,
            },
        }
    }

    pub(self) fn find_winner<'a>(
        &self,
        query: impl Iterator<Item = (&'a Player, &'a PlayerDamage)>,
        force: bool,
    ) -> MatchWinner {
        let winner = match self {
            Self::Score => Self::find_score_winner(query),
            Self::Stock(_) => Self::find_last_player_standing(query),
            Self::Stamina(_) => Self::find_last_player_standing(query),
        };
        if let MatchWinner::Undecided = winner {
            if force {
                MatchWinner::NoContest
            } else {
                MatchWinner::Undecided
            }
        } else {
            winner
        }
    }

    fn find_score_winner<'a>(
        query: impl Iterator<Item = (&'a Player, &'a PlayerDamage)>,
    ) -> MatchWinner {
        let mut winner = MatchWinner::Undecided;
        let mut max_score = i16::MIN;
        for (player, damage) in query {
            if let PlayerDamage::Score { score, .. } = damage {
                if *score > max_score {
                    winner = MatchWinner::Player(player.clone());
                    max_score = *score;
                }
            } else {
                panic!("Player in score match has non-score PlayerDamage component");
            }
        }
        winner
    }

    fn find_last_player_standing<'a>(
        query: impl Iterator<Item = (&'a Player, &'a PlayerDamage)>,
    ) -> MatchWinner {
        let mut winner = MatchWinner::Undecided;
        for (player, damage) in query {
            if damage.is_alive() {
                winner = match winner {
                    MatchWinner::Undecided => MatchWinner::Player(player.clone()),
                    _ => return MatchWinner::Undecided,
                };
            }
        }
        winner
    }
}

fn update_match_state(
    config: Res<MatchConfig>,
    mut state: ResMut<MatchState>,
    mut results: ResMut<MatchResult>,
    players: Query<(&Player, &PlayerDamage)>,
) {
    if let Some(ref mut time) = state.time_remaining {
        if *time == 0 {
            results.winner = config.rule.find_winner(players.iter(), /*force=*/ true);
        } else {
            *time -= 1;
        }
    }
}

fn on_player_died(
    mut events: EventReader<PlayerDied>,
    config: Res<MatchConfig>,
    mut results: ResMut<MatchResult>,
    players: Query<(&Player, &PlayerDamage)>,
) {
    let mut count = 0_u32;
    for event in events.iter() {
        info!("Player {} died: {:?}", event.player.id, event.damage);
        count += 1;
    }
    if count != 0 {
        results.winner = config.rule.find_winner(players.iter(), /*force=*/ false);
    }
}

// FIXME(james7132): Sending an AppExit is not a viable long term approach, fix this.
fn finish_match(result: Res<MatchResult>, mut exit: EventWriter<bevy::app::AppExit>) {
    match &result.winner {
        MatchWinner::Undecided => {}
        winner => {
            info!("Match finished: The winner is: {:?}", winner);
            exit.send(AppExit);
        }
    }
}

pub(super) fn build(builder: &mut AppBuilder) {
    builder.add_system_set(
        on_match_update()
            .with_system(update_match_state.system())
            .with_system(finish_match.system())
            .with_system(on_player_died.system()),
    );
}
