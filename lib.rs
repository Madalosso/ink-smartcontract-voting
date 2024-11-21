#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod voting {
    use core::cmp::Ordering;
    use ink::{prelude::vec::Vec, storage::Mapping};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Voting {
        runners: ink::prelude::vec::Vec<AccountId>,
        votes: ink::storage::Mapping<AccountId, u32>,
        already_voted: ink::storage::Mapping<AccountId, bool>,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum VoteError {
        AlreadyVoted,
        VoteOverflow,
    }

    impl Default for Voting {
        fn default() -> Self {
            Voting::new()
        }
    }

    impl Voting {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            let votes = Mapping::default();
            let already_voted = Mapping::default();
            let runners = Vec::new();
            Self {
                votes,
                already_voted,
                runners,
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        #[ink(message)]
        pub fn get_votes(&self, address: AccountId) -> u32 {
            self.votes.get(address).unwrap_or_default()
        }

        #[ink(message)]
        pub fn vote(&mut self, address: AccountId) -> Result<(), VoteError> {
            // check if caller already voted
            let caller = self.env().caller();
            if self.already_voted.get(caller).unwrap_or(false) {
                return Err(VoteError::AlreadyVoted);
            }

            // tag caller as already voted
            self.already_voted.insert(caller, &true);

            // store vote
            let current_votes = self.votes.get(address).unwrap_or_default();

            // if no votes yet, add address to runners
            if current_votes == 0 {
                self.runners.push(address);
            }

            // Issue: Potential overflow
            // Could use saturating_add so it wont return an error.
            match current_votes.checked_add(1) {
                Some(new_votes) => self.votes.insert(address, &new_votes),
                None => return Err(VoteError::VoteOverflow),
            };

            Ok(())
        }

        #[ink(message)]
        pub fn get_current_winner(&self) -> Vec<AccountId> {
            let mut current_winners = Vec::new();
            let mut highest_votes = 0;
            for runner in &self.runners {
                let votes = self.votes.get(*runner).unwrap_or(0);

                match votes.cmp(&highest_votes) {
                    Ordering::Greater => {
                        highest_votes = votes;
                        current_winners.clear();
                        current_winners.push(*runner)
                    }
                    Ordering::Equal => current_winners.push(*runner),
                    Ordering::Less => {}
                }
            }
            current_winners
        }
    }

    // TODO:
    // Write unitary tests
    // Write integration tests
    // e2e tests?
    // README file
    // upload github

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let voting = Voting::default();

            // assert current winners == []
            // assert runners.length == 0
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn accept_new_vote() {
            // How such unit test could "mock data"?
            // let mut voting = Voting::new();
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink::primitives::AccountId;
        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = VotingRef::default();

            // When
            let contract = client
                .instantiate("voting", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Voting>();

            // Then
            let alice_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);

            let get_alice_votes = call_builder.get_votes(alice_account);
            // let alice_votes = call_builder.get_votes(&ink_e2e::alice().public_key());
            let alice_votes = client
                .call(&ink_e2e::alice(), &get_alice_votes)
                .dry_run()
                .await?;
            assert!(matches!(alice_votes.return_value(), 0));

            Ok(())
        }

        // #[ink_e2e::test]
        // async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     // Given
        //     let mut constructor = VotingRef::new(false);
        //     let contract = client
        //         .instantiate("voting", &ink_e2e::bob(), &mut constructor)
        //         .submit()
        //         .await
        //         .expect("instantiate failed");
        //     let mut call_builder = contract.call_builder::<Voting>();

        //     let get = call_builder.get();
        //     let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        //     assert!(matches!(get_result.return_value(), false));

        //     // When
        //     let flip = call_builder.flip();
        //     let _flip_result = client
        //         .call(&ink_e2e::bob(), &flip)
        //         .submit()
        //         .await
        //         .expect("flip failed");

        //     // Then
        //     let get = call_builder.get();
        //     let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        //     assert!(matches!(get_result.return_value(), true));

        //     Ok(())
        // }
    }
}
