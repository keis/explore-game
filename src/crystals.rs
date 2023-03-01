use bevy::prelude::*;
use std::cmp::min;

#[derive(Component, Debug)]
pub struct CrystalDeposit {
    pub amount: u8,
}

impl CrystalDeposit {
    pub fn take(&mut self) -> u8 {
        let take = min(self.amount, 10);
        self.amount -= take;
        take
    }
}
