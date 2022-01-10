use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_cure(&mut self) {
        // get the sender address
        let sender_id = env::predecessor_account_id();
        let burn_address: AccountId = AccountId::new_unchecked("burn.near".to_string());

        // get a token to cure
        let tokens = self.tokens_per_owner.get(&sender_id).expect("Account not infected.").to_vec();

        tokens.iter()
            .map(|token_id| self.internal_transfer(
                &sender_id,
                &burn_address,
                &token_id,
                None,
                None))
            .for_each(drop);

        //console log confirming that the account has been cured
        env::log_str("Cured of thevarus");
    }

}