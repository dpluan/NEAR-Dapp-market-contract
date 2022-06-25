use crate::*;

const GAS_FOR_NFT_USES: Gas = 15_000_000_000_000;

#[ext_contract(nft_contract)]
pub trait NFTContract {
    fn nft_use_payout(
        &mut self,
        user_id: AccountId,
        token_id: TokenId,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[ext_contract(ext_self)]
pub trait MarketContract {
    fn resolve_use(&mut self, user_id: AccountId, price: U128) -> Promise;
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn apply_use(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");
        let contract_and_token_id = format!(
            "{}{}{}",
            nft_contract_id.clone(),
            DELIMETER,
            token_id.clone()
        );

        let uses = self
            .uses
            .get(&contract_and_token_id)
            .expect("Not found uses");

        let user_id = env::predecessor_account_id();
        assert_ne!(user_id, uses.owner_id, "Can not use your own contract");
        let price = uses.use_conditions.0;
        assert!(
            deposit >= price,
            "Attached deposit must be greater than or equal current price: {}",
            price
        );
        self.process_uses(nft_contract_id, token_id, U128(price), user_id);
    }

    #[private]
    pub fn process_uses(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        price: U128,
        user_id: AccountId,
    ) {
        nft_contract::nft_use_payout(
            user_id.clone(),
            token_id,
            "Payout for use nft from market_contract".to_string(),
            price,
            10,
            &nft_contract_id,
            1,
            GAS_FOR_NFT_USES,
        )
        .then(ext_self::resolve_use(
            user_id,
            price,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ));
    }

    pub fn resolve_use(&mut self, user_id: AccountId, price: U128) -> U128 {
        self.internal_payout(user_id, price)
    }

    #[payable]
    pub fn update_use_price(&mut self, nft_contract_id: AccountId, token_id: TokenId, price: U128) {
        assert_one_yocto();
        let contract_and_token_id = format!(
            "{}{}{}",
            nft_contract_id.clone(),
            DELIMETER,
            token_id.clone()
        );
        let mut uses = self
            .uses
            .get(&contract_and_token_id)
            .expect("Not found uses");
        assert_eq!(
            env::predecessor_account_id(),
            uses.owner_id,
            "Must be sale owner"
        );
        uses.use_conditions = price;
        self.uses.insert(&contract_and_token_id, &uses);
    }

    #[payable]
    pub fn remove_uses(&mut self, nft_contract_id: AccountId, token_id: TokenId) {
        assert_one_yocto();
        let uses = self.internal_remove_uses(nft_contract_id, token_id);
        assert_eq!(
            env::predecessor_account_id(),
            uses.owner_id,
            "Must be owner id"
        );
    }
}
