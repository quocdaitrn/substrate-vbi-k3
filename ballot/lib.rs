#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ballot {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
        traits::PackedLayout,
        traits::SpreadLayout,
    };
    use scale::{Encode, Decode};
    use scale_info::TypeInfo;
    use ink_prelude::vec::Vec;

    #[derive(TypeInfo, SpreadLayout, PackedLayout, Encode, Decode, Clone)]
    pub struct Voter {
        pub has_right_to_vote: bool, // true if the voter has right to vote
        pub voted: bool, // if true, that person already voted
        pub vote: u64, // index of the voted proposal
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if caller is not chairperson while required to.
        CallerIsNotChairPerson,
        /// Returned if caller has already voted.
        CallerHasVoted,
        /// Returned if caller has not authorized to vote.
        CallerHasNotAuthorizedToVote,
        /// Returned if the voter is not found.
        VoterNotFound,
        /// Returned if the proposal is not found.
        ProposalNotFound,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Emitted whenever a new vote occurs.
    #[ink(event)]
    pub struct Vote {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        proposal_index: u64,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct Ballot {
        voters: Mapping<AccountId, Voter>,
        proposals: Vec<u64>,
        chairperson: AccountId,
    }

    impl Ballot {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();

            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.voters = Default::default();
                contract.voters.insert(&caller, &Voter {
                    has_right_to_vote: true,
                    voted: false,
                    vote: Default::default(),
                });

                contract.proposals = Vec::new();

                contract.chairperson = caller;
            })
        }

        #[ink(message)]
        pub fn get_chairperson(&self) -> AccountId {
            self.chairperson
        }

        #[ink(message)]
        pub fn give_right_to_vote(&mut self, voter: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.chairperson {
                return Err(Error::CallerIsNotChairPerson)
            }

            self.voters.insert(&voter, &Voter {
                has_right_to_vote: true,
                voted: false,
                vote: Default::default(),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn init_proposals(&mut self, num: u64) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.chairperson {
                return Err(Error::CallerIsNotChairPerson)
            }

            for _ in 0..num {
                self.proposals.push(0);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn get_voter(&self, account: AccountId) -> Result<Voter> {
            let voter_option = self.voters.get(&account);
            match voter_option {
                Some(voter) => Ok(voter.clone()),
                None => Err(Error::VoterNotFound),
            }
        }

        #[ink(message)]
        pub fn vote(&mut self, proposal: u64) -> Result<()> {
            let caller = self.env().caller();
            let voter_option = self.voters.get(&caller);
            if voter_option.is_none() {
                return Err(Error::CallerHasNotAuthorizedToVote)
            }
            let voter = voter_option.unwrap();
            if !voter.has_right_to_vote {
                return Err(Error::CallerHasNotAuthorizedToVote)
            }
            if voter.voted {
                return Err(Error::CallerHasVoted)
            }

            if proposal >= self.proposals.len() as u64 {
                return Err(Error::ProposalNotFound)
            }

            self.voters.insert(&caller, &Voter {
                has_right_to_vote: true,
                voted: true,
                vote: proposal.clone(),
            });

            self.proposals[proposal as usize] += 1;

            self.env().emit_event(Vote { account: caller.clone(), proposal_index: proposal.clone() });

            Ok(())
        }

        #[ink(message)]
        pub fn get_proposal_votes(&self, proposal: u64) -> Result<u64> {
            if proposal >= self.proposals.len() as u64 {
                return Err(Error::ProposalNotFound)
            }

            Ok(self.proposals[proposal as usize].clone())
        }

        #[ink(message)]
        pub fn winning_proposal(&self) -> u64 {
            let mut max = 0;
            let mut max_index = 0;
            for (i, proposal) in self.proposals.iter().enumerate() {
                if *proposal > max {
                    max = *proposal;
                    max_index = i;
                }
            }
            max_index as u64
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<Environment>()
        }

        fn set_next_caller(caller: AccountId) {
            ink_env::test::set_caller::<Environment>(caller);
        }

        /// We test if the default constructor does its job.
        #[ink::test]
        fn constructor_works() {
            let default_accounts = default_accounts();

            set_next_caller(default_accounts.alice);

            let contract = Ballot::new();
            assert_eq!(contract.chairperson, default_accounts.alice);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn get_chairperson_works() {
            let default_accounts = default_accounts();

            set_next_caller(default_accounts.alice);

            let contract = Ballot::new();
            let chairperson = contract.get_chairperson();
            assert_eq!(chairperson, default_accounts.alice);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn give_right_to_vote_works() {
            let default_accounts = default_accounts();

            set_next_caller(default_accounts.alice);

            let mut contract = Ballot::new();
            let result = contract.give_right_to_vote(default_accounts.bob);

            assert_eq!(result, Ok(()));
            assert_eq!(contract.voters.get(default_accounts.bob).unwrap().has_right_to_vote, true);
        }
    }
}
