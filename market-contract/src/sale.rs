use std::collections::HashMap;

use crate::*;

pub const GAS_FOR_ROYALTIES: Gas = 115_000_000_000_000;
const GAS_FOR_NFT_TRANSFER: Gas = 15_000_000_000_000;
pub const NO_DEPOSIT: Balance = 0;
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[ext_contract(nft_contract)]
pub trait NFTContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[ext_contract(ext_self)]
pub trait MarketContract {
    fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> Promise;
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        assert_one_yocto();

        let sale = self.internal_remove_sale(nft_contract_id, token_id);
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be owner id"
        );
    }

    #[payable]
    pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: TokenId, price: U128) {
        assert_one_yocto();
        let contract_and_token_id = format!(
            "{}{}{}",
            nft_contract_id.clone(),
            DELIMETER,
            token_id.clone()
        );
        let mut sale = self
            .sales
            .get(&contract_and_token_id)
            .expect("Not found sale");
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        sale.sale_conditions = price;
        self.sales.insert(&contract_and_token_id, &sale);
    }

    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");
        let contract_and_token_id = format!(
            "{}{}{}",
            nft_contract_id.clone(),
            DELIMETER,
            token_id.clone()
        );
        let sale = self
            .sales
            .get(&contract_and_token_id)
            .expect("Not found sale");
        let buyer_id = env::predecessor_account_id();
        assert_ne!(buyer_id, sale.owner_id, "Can not bid on your own sale");

        let price = sale.sale_conditions.0;
        assert!(
            deposit >= price,
            "Attached deposit must be greater than or equal current price: {}",
            price
        );

        self.process_purchase(nft_contract_id, token_id, U128(deposit), buyer_id);
    }

    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        price: U128,
        buyer_id: AccountId,
    ) -> Promise {
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());
        self.internal_remove_uses(nft_contract_id.clone(), token_id.clone());
        nft_contract::nft_transfer_payout(
            buyer_id.clone(),
            token_id,
            sale.approval_id,
            "Payout from market contract".to_string(),
            price,
            10,
            &nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_purchase(
            buyer_id,
            price,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }

    pub fn resolve_purchase(&mut self, buyer_id: AccountId, price: U128) -> U128 {
        self.internal_payout(buyer_id, price)
    }
}
