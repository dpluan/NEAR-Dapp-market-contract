use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_supply_uses(&self) -> U128 {
        U128(self.uses.len() as u128)
    }

    pub fn get_uses(&self, from_index: Option<u128>, limit: Option<u64>) -> Vec<Uses> {
        let start = u128::from(from_index.unwrap_or(0));
        self.uses
            .values()
            .skip(start as usize)
            .take(limit.unwrap_or(0) as usize)
            .collect()
    }
}
