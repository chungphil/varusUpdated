use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;
mod cure;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,

    //keeps track of vaxxed participants
    pub vaxxxed: UnorderedSet<AccountId>,

    //index number for tokens
    pub next_token_id: TokenId,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    Vaxxxed,
    NextTokenId,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        //calls the other function "new: with some default metadata and the owner_id passed in 
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "thevarus2022".to_string(),
                symbol: "VARUS".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        //create a variable of type Self with all the fields initialized. 
        let this = Self {
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            //set the owner_id field equal to the passed in owner_id. 
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            vaxxxed: UnorderedSet::new(StorageKey::Vaxxxed.try_to_vec().unwrap(),),
            next_token_id: 0
        };

        //return the Contract object
        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    /// Create a virtual blockchain from input parameters
    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: "contract.testnet".to_string(),
            signer_account_id: alice().to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 10u128.pow(25) as Balance,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 10u128.pow(24) as Balance,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    /// Helper function to create TokenMetadata of thevarus
    fn get_thevarus() -> TokenMetadata {
        TokenMetadata {
            title: Some(String::from("thevarus")),
            description: Some(String::from("pathogen")),
            media: Some(String::from("https://tinyurl.com/bddjmwk4")),
            media_hash: Some(Base64VecU8(vec![0,1,2])),
            copies: Some(1),
            issued_at: Some(1_000),
            expires_at: Some(1_000_000),
            starts_at: Some(10_000),
            updated_at: Some(100_000),
            extra: Some(String::from("some extra data")),
            reference: Some(String::from("thevarus.extra-info")),
            reference_hash: Some(Base64VecU8(vec![1,2,3])),
        }
    }

    /// Helper function to construct a valid account from input string
    pub fn contract() -> AccountId {
        AccountId::new_unchecked("contract.near".to_string())
    }

    /// Helper function to construct the burn address
    fn burn() -> AccountId {
        AccountId::new_unchecked("burn.near".to_string())
    }

    /// Helper function to construct the original TokenID
    fn original() -> TokenId {
        0u64
    }

    /// Helper function to construct the mutant TokenID
    fn mutant() -> TokenId {
        1u64
    }

    /// Ensure initialisation of metadata works and that the vaxxx list begins empty
    #[test]
    fn check_initialisation() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());
        assert_eq!(0, contract.vaxxxed.len(), "Expected vaxxxed to be an empty vector.");

        let option = contract.metadata.take().unwrap();
        assert_eq!("nft-1.0.0", option.spec, "Expected different spec.");
        assert_eq!("thevarus2022", option.name, "Expected different name.");
        assert_eq!("VARUS",option.symbol,"Expected different symbol.");
    }

    ////////////////////
    //// Mint Tests ////
    ////////////////////

    /// Ensure that minting without providing a receiver id sends the NFT to the caller
    #[test]
    fn mint_no_receiver() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        let token = contract.tokens_by_id.get(&original()).unwrap();
        assert_eq!(alice(), token.owner_id, "Token should belong to alice.");
    }

    /// Ensure that minting and providing a receiver id sends the NFT to the receiver
    #[test]
    fn mint_with_receiver() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        contract.nft_mint(
            get_thevarus(),
            bob(),
            None
        );

        let token = contract.tokens_by_id.get(&original()).unwrap();
        assert_eq!(bob(), token.owner_id, "Token should belong to bob.");
    }

    /// Ensure that metadata of a minted token is correct
    #[test]
    fn mint_check_metadata() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        let actual = contract.token_metadata_by_id.get(&original()).unwrap();
        let expected = get_thevarus();
        assert_eq!(expected.title, actual.title, "Expected title to be equal.");
        assert_eq!(expected.description, actual.description, "Expected description to be equal.");
        assert_eq!(expected.media, actual.media, "Expected media to be equal.");
        assert_eq!(expected.media_hash, actual.media_hash, "Expected media_hash to be equal.");
        assert_eq!(expected.copies, actual.copies, "Expected copies to be equal.");
        assert_eq!(expected.issued_at, actual.issued_at, "Expected issued-at to be equal.");
        assert_eq!(expected.expires_at, actual.expires_at, "Expected expires_at to be equal.");
        assert_eq!(expected.starts_at, actual.starts_at, "Expected starts_at to be equal.");
        assert_eq!(expected.updated_at, actual.updated_at, "Expected updated_at to be equal.");
        assert_eq!(expected.extra, actual.extra, "Expected actual to be equal.");
        assert_eq!(expected.reference, actual.reference, "Expected reference to be equal.");
        assert_eq!(expected.reference_hash, actual.reference_hash, "Expected reference_hash to be equal.");
    }

    ////////////////////
    //// Cure Tests ////
    ////////////////////

    /// Test cure for a single token
    #[test]
    fn check_cure() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        assert_eq!(U128::from(0), contract.nft_supply_for_owner(alice()), "Alice should not be infected yet.");

        // Mint a token
        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        // Get the minted token
        let token = contract.tokens_by_id.get(&original()).unwrap();

        assert_eq!(alice(), token.owner_id, "Token should belong to alice after transfer.");
        assert_eq!(U128::from(1), contract.nft_supply_for_owner(alice()), "Alice should be infected.");

        // Cure self
        contract.nft_cure();

        let token = contract.tokens_by_id.get(&original()).unwrap();
        assert_eq!(burn(), token.owner_id, "Token should belong to burn after transfer.");
        assert_eq!(U128::from(0), contract.nft_supply_for_owner(alice()), "Alice should no longer be infected.");
    }

    /// Test cure for multiple tokens
    #[test]
    fn check_multi_cure() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        assert_eq!(U128::from(0), contract.nft_supply_for_owner(alice()), "Alice should not be infected yet.");

        // Mint tokens
        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        // Get the minted token
        let token1 = contract.tokens_by_id.get(&original()).unwrap();
        let token2 = contract.tokens_by_id.get(&mutant()).unwrap();

        assert_eq!(alice(), token1.owner_id, "Token should belong to alice after transfer.");
        assert_eq!(alice(), token2.owner_id, "Token should belong to alice after transfer.");
        assert_eq!(U128::from(2), contract.nft_supply_for_owner(alice()), "Alice should be infected.");

        // Cure self
        contract.nft_cure();

        // Get the minted token
        let cured1 = contract.tokens_by_id.get(&original()).unwrap();
        let cured2 = contract.tokens_by_id.get(&mutant()).unwrap();
        assert_eq!(burn(), cured1.owner_id, "Token should belong to burn after transfer.");
        assert_eq!(burn(), cured2.owner_id, "Token should belong to burn after transfer.");
        assert_eq!(U128::from(0), contract.nft_supply_for_owner(alice()), "Alice should no longer be infected.");
    }


    /// Test cure panics if a user is not infected
    #[test]
    #[should_panic]
    fn cure_panics_non_infected() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        // Cure self
        contract.nft_cure();
    }

    ////////////////////////
    //// Transfer Tests ////
    ////////////////////////

    /// Ensure that the transfer sends the original token to the recipient
    #[test]
    fn transfer_sends_original() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        contract.nft_transfer(
            bob(),
            carol(),
            original(),
            None,
            None
        );

        let token = contract.tokens_by_id.get(&original()).unwrap();
        assert_eq!(bob(), token.owner_id, "Token should belong to bob after transfer.");
    }

    /// Ensure that the transfer sends the original token to the recipient
    #[test]
    fn transfer_creates_mutant() {
        let context = get_context(alice().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());

        contract.nft_mint(
            get_thevarus(),
            alice(),
            None
        );

        contract.nft_transfer(
            bob(),
            carol(),
            original(),
            None,
            None
        );

        let token = contract.tokens_by_id.get(&mutant()).unwrap();
        assert_eq!(carol(), token.owner_id, "Token should belong to bob after transfer.");
    }

    /////////////////////
    //// Vaxxx Tests ////
    /////////////////////

    /// Check that vaxxx function adds to the vaxxxed list
    #[test]
    fn vaxxx_adds_to_vaxxxed() {
        let context = get_context(bob().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());
        assert_eq!(0, contract.vaxxxed.len(), "Expected empty vaxxx list."); // Sanity check

        // vaxxx alice and bob
        contract.vaxxx(alice());
        contract.vaxxx(bob());

        // ensure vaxxx list now contains both alice and bob
        assert_eq!(2, contract.vaxxxed.len(), "Expected single addition to vaxxx list.");
        contract.vaxxxed.contains(&alice());
        contract.vaxxxed.contains(&bob());
    }

    /// Check that vaxxx_pass returns true for vaxxxed addresses and false for un-vaxxxed
    #[test]
    fn check_vaxxx_pass() {
        let context = get_context(bob().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());
        assert_eq!(0, contract.vaxxxed.len(), "Expected empty vaxxx list."); // Sanity check

        contract.vaxxx(alice()); // Vaxxx alice
        assert!(contract.vaxxx_pass(alice()), "Expected alice to be vaxxxed");
        assert!(!contract.vaxxx_pass(bob()), "Expected bob to be un-vaxxxed");
    }

    /// Check that the vaxxx_list contains all of the added addresses
    #[test]
    fn check_vaxxx_list() {
        let context = get_context(bob().to_string(), 0);
        testing_env!(context);
        let mut contract = Contract::new_default_meta(contract());
        assert_eq!(0, contract.vaxxxed.len(), "Expected empty vaxxx list."); // Sanity check

        // Vaxxx alice and bob
        contract.vaxxx(alice());
        contract.vaxxx(bob());

        // Check vaxxx_list
        let vaxxxed_vector = contract.vaxxx_list();
        assert_eq!("alice.near", vaxxxed_vector.get(0).unwrap().to_string(), "");
        assert_eq!("bob.near", vaxxxed_vector.get(1).unwrap().to_string(), "");
    }

}

