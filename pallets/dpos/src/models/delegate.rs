use crate::{BalanceOf, Config};
pub struct Delegation<T: Config> {
    pub amount: BalanceOf<T>,
}

impl<T: Config> Delegation<T> {
    pub fn new(amount: BalanceOf<T>) -> Self {
        Self { amount }
    }

    pub fn set_amount(&mut self, amount: BalanceOf<T>) {
        self.amount = amount;
    }
}