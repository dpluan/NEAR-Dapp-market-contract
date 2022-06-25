use crate::*;

#[near_bindgen]
impl Contract {
    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {
        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
        let sale = self
            .sales
            .remove(&contract_and_token_id)
            .expect("Not found sale");
        let mut by_owner_id = self
            .by_owner_id
            .get(&sale.owner_id)
            .expect("Not found sale by owner");
        by_owner_id.remove(&contract_and_token_id);
        self.by_owner_id.insert(&sale.owner_id, &by_owner_id);
        if by_owner_id.is_empty() {
            self.by_owner_id.remove(&sale.owner_id);
        } else {
            self.by_owner_id.insert(&sale.owner_id, &by_owner_id);
        };

        let mut by_contract_id = self
            .by_contract_id
            .get(&nft_contract_id)
            .expect("Not found sale by contract_id");

        by_contract_id.remove(&token_id);
        if by_contract_id.is_empty() {
            self.by_contract_id.remove(&nft_contract_id);
        } else {
            self.by_contract_id
                .insert(&nft_contract_id, &by_contract_id);
        }

        sale
    }

    pub(crate) fn internal_remove_uses(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Uses {
        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
        let uses = self
            .uses
            .remove(&contract_and_token_id)
            .expect("Not found uses");
        uses
    }

    pub(crate) fn internal_payout(&mut self, buyer_id: AccountId, price: U128) -> U128 {
        let payout_option = promise_result_as_success().and_then(|value| {
            let payout_object =
                near_sdk::serde_json::from_slice::<Payout>(&value).expect("Invalid payout object");
            if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
                env::log("Cannot have more than 10 royalities".as_bytes());
                None
            } else {
                let mut remainder = price.0;
                for &value in payout_object.payout.values() {
                    remainder = remainder.checked_sub(value.0)?;
                }

                if remainder == 0 || remainder == 1 {
                    Some(payout_object.payout)
                } else {
                    None
                }
            }
        });

        let payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
            Promise::new(buyer_id).transfer(u128::from(price));
            return price;
        };

        for (receiver_id, amount) in payout {
            Promise::new(receiver_id).transfer(amount.into());
        }
        price
    }
}
