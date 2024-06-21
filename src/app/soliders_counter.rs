use super::{Player, ROW_WIDTH};
use leptos::prelude::*;

const PLAYER_SOLIDERS_INIT: usize = (ROW_WIDTH * ROW_WIDTH) / 2;

#[derive(Clone, Copy)]
pub struct SolidersCounter {
    p1: RwSignal<usize>,
    p2: RwSignal<usize>,
}

impl SolidersCounter {
    pub fn new() -> Self {
        let (p1, p2) = (
            RwSignal::new(PLAYER_SOLIDERS_INIT),
            RwSignal::new(PLAYER_SOLIDERS_INIT),
        );
        Self { p1, p2 }
    }

    pub fn reset(&self) {
        self.p1.set(PLAYER_SOLIDERS_INIT);
        self.p2.set(PLAYER_SOLIDERS_INIT);
    }

    pub fn get(&self, player: Player) -> RwSignal<usize> {
        match player {
            Player::One => self.p1,
            Player::Two => self.p2,
        }
    }
}
