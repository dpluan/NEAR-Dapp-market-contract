use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, env::STORAGE_PRICE_PER_BYTE, ext_contract, near_bindgen, promise_result_as_success,
    AccountId, Balance, CryptoHash, Gas, PanicOnDefault, Promise,
};

use crate::deploy::*;
use crate::internal::*;
use crate::nft_callback::*;
use crate::sale::*;
use crate::sale_view::*;
use crate::uses::*;
use crate::uses_view::*;
use crate::utils::*;

mod deploy;
mod internal;
mod nft_callback;
mod sale;
mod sale_view;
mod uses;
mod uses_view;
mod utils;

const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
static DELIMETER: &str = ".";

pub type TokenId = String;
pub type NFTContractId = String;
pub type SalePriceInYoctoNear = U128;
pub type UsePriceInYoctoNear = U128;
pub type ContractAndTokenId = String; // nft-tutorial.vbi.dev.testnet.VBI_NFT#01

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: NFTContractId,
    pub token_id: TokenId,
    pub sale_conditions: SalePriceInYoctoNear,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Uses {
    pub owner_id: AccountId,
    pub nft_contract_id: NFTContractId,
    pub token_id: TokenId,
    pub use_conditions: UsePriceInYoctoNear,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DeployedSmartContract {
    pub contract_deploy_address: AccountId,
    pub frontend_address: String,
    pub contract_name: String,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    // Uses:
    pub creates: LookupMap<AccountId, UnorderedSet<DeployedSmartContract>>,
    pub uses: UnorderedMap<ContractAndTokenId, Uses>,
    // Sales
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    pub by_contract_id: LookupMap<NFTContractId, UnorderedSet<TokenId>>,
    pub storage_deposit: LookupMap<AccountId, Balance>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum StorageKey {
    SaleKey,
    UsesKey,
    CreateKey,
    InnerByCreatorIdKey { account_id_hash: CryptoHash },
    ByOwnerIdKey,
    InnerByOwnerIdKey { account_id_hash: CryptoHash },
    ByContractIdKey,
    InnerByContractIdKey { account_id_hash: CryptoHash },
    StorageDepositKey,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            sales: UnorderedMap::new(StorageKey::SaleKey.try_to_vec().unwrap()),
            uses: UnorderedMap::new(StorageKey::UsesKey.try_to_vec().unwrap()),
            creates: LookupMap::new(StorageKey::CreateKey.try_to_vec().unwrap()),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerIdKey.try_to_vec().unwrap()),
            by_contract_id: LookupMap::new(StorageKey::ByContractIdKey.try_to_vec().unwrap()),
            storage_deposit: LookupMap::new(StorageKey::StorageDepositKey.try_to_vec().unwrap()),
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id.unwrap_or(env::predecessor_account_id());
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Require deposit minimum of {}",
            STORAGE_PER_SALE
        );

        let mut balance = self.storage_deposit.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposit.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let amount = self.storage_deposit.remove(&owner_id).unwrap_or(0);
        let sales = self.by_owner_id.get(&owner_id);
        let len = sales.map(|s| s.len()).unwrap_or_default();
        let storage_required = u128::from(len) * STORAGE_PER_SALE;
        assert!(amount >= storage_required);
        let diff = amount - storage_required;
        if diff > 0 {
            Promise::new(owner_id.clone()).transfer(diff);
        }

        if storage_required > 0 {
            self.storage_deposit.insert(&owner_id, &storage_required);
        }
    }

    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposit.get(&account_id).unwrap_or(0))
    }
}
