#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod wasmerc20 {
    use ink_storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Wasmerc20 {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        approval: Mapping<(AccountId, AccountId), Balance>,
        owner: AccountId,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        owner: AccountId,
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientApproval,
        IllegalManager,
    }

    impl Wasmerc20 {
        /// Constructor that initializes.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let sender = Self::env().caller();
            balances.insert(&sender, &total_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(sender),
                value: total_supply,
            });

            Self {
                total_supply,
                balances,
                approval: Default::default(),
                owner: sender,
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, who: AccountId) -> Balance {
            self.balances.get(&who).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approval(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.approval.get(&(owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
        ) -> core::result::Result<(), Error> {
            let from = self.env().caller();
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self._transfer(Some(from), Some(to), value)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let approval = self.approval(from, caller);
            if approval < value {
                return Err(Error::InsufficientApproval);
            }

            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.approval.insert((from, caller), &(approval - value));
            self._transfer(Some(from), Some(to), value)
        }

        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
            let owner = self.env().caller();
            self.approval.insert((owner, to), &value);

            self.env().emit_event(Approval {
                owner,
                spender: to,
                value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn mint(
            &mut self,
            value: Balance,
        ) -> core::result::Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::IllegalManager);
            }

            self.total_supply += value;
            self._transfer(None, Some(caller), value)
        }

        #[ink(message)]
        pub fn burn(
            &mut self,
            value: Balance,
        ) -> core::result::Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::IllegalManager);
            }

            let caller_balance = self.balance_of(caller);
            if caller_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.total_supply -= value;
            self._transfer(Some(caller), None, value)
        }

        pub fn _transfer(
            &mut self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            value: Balance,
        ) -> Result<(), Error> {
            if from.is_some() {
                let from_balance = self.balance_of(from.unwrap());
                self.balances.insert(&from.unwrap(), &(from_balance - value));
            }
            
            if to.is_some() {
                let to_balance = self.balance_of(to.unwrap());
                self.balances.insert(&to.unwrap(), &(to_balance + value));
            }
            
            self.env().emit_event(Transfer {
                from,
                to,
                value,
            });

            Ok(())
        }
    }

}
