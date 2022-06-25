use crate::*;

const INITIAL_BALANCE: Balance = 3_000_000_000_000_000_000_000_000; // 3e24yN, 3N

#[near_bindgen]
impl Contract {
    #[private]
    pub fn create_child_contract(prefix: AccountId, code: Vec<u8>) -> Promise {
        let subaccount_id =
            AccountId::try_from(format!("{}.{}", prefix, env::current_account_id()));
        Promise::new(subaccount_id.unwrap())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(INITIAL_BALANCE)
            .deploy_contract(code.to_vec())
    }

    #[private]
    pub fn create_smart_contract(
        &mut self,
        creator_id: AccountId,
        contract_deploy_address: AccountId,
        frontend_address: String,
        contract_name: String,
    ) {
        let deployed_smart_contract = DeployedSmartContract {
            contract_deploy_address,
            frontend_address,
            contract_name,
        };

        let mut creates = self.creates.get(&creator_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::InnerByCreatorIdKey {
                    account_id_hash: hash_account_id(&creator_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        creates.insert(&deployed_smart_contract);
        self.creates.insert(&creator_id, &creates);
    }

    pub fn get_created_contract_by_creator(
        &self,
        creator_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DeployedSmartContract> {
        let by_creator_id = self.creates.get(&creator_id);
        let deploy_smart_contracts = if let Some(by_creator_id) = by_creator_id {
            by_creator_id
        } else {
            return vec![];
        };
        let start = u128::from(from_index.unwrap_or(0));
        deploy_smart_contracts
            .as_vector()
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(0) as usize)
            .collect()
    }
}
