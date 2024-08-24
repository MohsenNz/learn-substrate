#![cfg_attr(not(feature = "std"), no_std)]

// use frame::prelude::*;

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

// #[frame::pallet]
// pub mod pallet {
//     use super::*;

//     #[pallet::config]
//     pub trait Config: frame_system::Config {}

//     #[pallet::pallet]
//     pub struct Pallet<T>(_);
// }

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;

    pub type Balance = u128;

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Single storage item, of type `Balance`.
    #[pallet::storage]
    pub type TotalIssuance<T: Config> = StorageValue<_, Balance>;

    /// A mapping from `T::AccountId` to `Balance`
    #[pallet::storage]
    pub type Balances<T: Config> = StorageMap<_, _, T::AccountId, Balance>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An unsafe mint that can be called by anyone. Not a great idea.
        pub fn mint_unsafe(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            // ensure that this is a signed account, but we don't really check `_anyone`.
            let _anyone = ensure_signed(origin)?;

            // update the balances map. Notice how all `<T: Config>` remains as `<T>`.
            Balances::<T>::mutate(dest, |b| *b = Some(b.unwrap_or(0) + amount));
            // update total issuance.
            TotalIssuance::<T>::mutate(|t| *t = Some(t.unwrap_or(0) + amount));

            Ok(())
        }

        /// Transfer `amount` from `origin` to `dest`.
        pub fn transfer(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // ensure sender has enough balance, and if so, calculate what is left after `amount`.
            let sender_balance = Balances::<T>::get(&sender).ok_or("NonExistentAccount")?;
            if sender_balance < amount {
                return Err("InsufficientBalance".into());
            }
            let reminder = sender_balance - amount;

            // update sender and dest balances.
            Balances::<T>::mutate(dest, |b| *b = Some(b.unwrap_or(0) + amount));
            Balances::<T>::insert(&sender, reminder);

            Ok(())
        }
    }

    #[allow(unused)]
    impl<T: Config> Pallet<T> {
        pub fn transfer_better(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let sender_balance = Balances::<T>::get(&sender).ok_or("NonExistentAccount")?;
            ensure!(sender_balance >= amount, "InsufficientBalance");
            let reminder = sender_balance - amount;

            // .. snip
            Ok(())
        }

        /// Transfer `amount` from `origin` to `dest`.
        pub fn transfer_better_checked(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let sender_balance = Balances::<T>::get(&sender).ok_or("NonExistentAccount")?;
            let reminder = sender_balance
                .checked_sub(amount)
                .ok_or("InsufficientBalance")?;

            // .. snip
            Ok(())
        }
    }

    #[cfg(any(test, doc))]
    pub(crate) mod tests {
        use crate::guides::your_first_pallet::pallet::*;

        use frame::testing_prelude::*;

        pub(crate) const ALICE: u64 = 1;
        pub(crate) const BOB: u64 = 2;
        pub(crate) const CHARLIE: u64 = 3;

        // This runtime is only used for testing, so it should be somewhere like `#[cfg(test)] mod
        // tests { .. }`
        mod runtime {
            use super::*;
            // we need to reference our `mod pallet` as an identifier to pass to
            // `construct_runtime`.
            // YOU HAVE TO CHANGE THIS LINE BASED ON YOUR TEMPLATE
            use crate::guides::your_first_pallet::pallet as pallet_currency;

            construct_runtime!(
                pub enum Runtime {
                    // ---^^^^^^ This is where `enum Runtime` is defined.
                    System: frame_system,
                    Currency: pallet_currency,
                }
            );

            #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
            impl frame_system::Config for Runtime {
                type Block = MockBlock<Runtime>;
                // within pallet we just said `<T as frame_system::Config>::AccountId`, now we
                // finally specified it.
                type AccountId = u64;
            }

            // our simple pallet has nothing to be configured.
            impl pallet_currency::Config for Runtime {}
        }

        pub(crate) use runtime::*;

        #[allow(unused)]
        fn new_test_state_basic() -> TestState {
            let mut state = TestState::new_empty();
            let accounts = vec![(ALICE, 100), (BOB, 100)];
            state.execute_with(|| {
                for (who, amount) in &accounts {
                    Balances::<Runtime>::insert(who, amount);
                    TotalIssuance::<Runtime>::mutate(|b| *b = Some(b.unwrap_or(0) + amount));
                }
            });

            state
        }

        pub(crate) struct StateBuilder {
            balances: Vec<(<Runtime as frame_system::Config>::AccountId, Balance)>,
        }

        impl Default for StateBuilder {
            fn default() -> Self {
                Self {
                    balances: vec![(ALICE, 100), (BOB, 100)],
                }
            }
        }

        impl StateBuilder {
            fn add_balance(
                mut self,
                who: <Runtime as frame_system::Config>::AccountId,
                amount: Balance,
            ) -> Self {
                self.balances.push((who, amount));
                self
            }
        }

        impl StateBuilder {
            pub(crate) fn build_and_execute(self, test: impl FnOnce() -> ()) {
                let mut ext = TestState::new_empty();
                ext.execute_with(|| {
                    for (who, amount) in &self.balances {
                        Balances::<Runtime>::insert(who, amount);
                        TotalIssuance::<Runtime>::mutate(|b| *b = Some(b.unwrap_or(0) + amount));
                    }
                });

                ext.execute_with(test);

                // assertions that must always hold
                ext.execute_with(|| {
                    assert_eq!(
                        Balances::<Runtime>::iter().map(|(_, x)| x).sum::<u128>(),
                        TotalIssuance::<Runtime>::get().unwrap_or_default()
                    );
                })
            }
        }

        #[test]
        fn first_test() {
            TestState::new_empty().execute_with(|| {
                // We expect Alice's account to have no funds.
                assert_eq!(Balances::<Runtime>::get(&ALICE), None);
                assert_eq!(TotalIssuance::<Runtime>::get(), None);

                // mint some funds into Alice's account.
                assert_ok!(Pallet::<Runtime>::mint_unsafe(
                    RuntimeOrigin::signed(ALICE),
                    ALICE,
                    100
                ));

                // re-check the above
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(100));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(100));
            })
        }

        #[test]
        fn state_builder_works() {
            StateBuilder::default().build_and_execute(|| {
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(100));
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(100));
                assert_eq!(Balances::<Runtime>::get(&CHARLIE), None);
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(200));
            });
        }

        #[test]
        fn state_builder_add_balance() {
            StateBuilder::default()
                .add_balance(CHARLIE, 42)
                .build_and_execute(|| {
                    assert_eq!(Balances::<Runtime>::get(&CHARLIE), Some(42));
                    assert_eq!(TotalIssuance::<Runtime>::get(), Some(242));
                })
        }

        #[test]
        #[should_panic]
        fn state_builder_duplicate_genesis_fails() {
            StateBuilder::default()
                .add_balance(CHARLIE, 42)
                .add_balance(CHARLIE, 43)
                .build_and_execute(|| {
                    assert_eq!(Balances::<Runtime>::get(&CHARLIE), None);
                    assert_eq!(TotalIssuance::<Runtime>::get(), Some(242));
                })
        }

        #[test]
        fn mint_works() {
            StateBuilder::default().build_and_execute(|| {
                // given the initial state, when:
                assert_ok!(Pallet::<Runtime>::mint_unsafe(
                    RuntimeOrigin::signed(ALICE),
                    BOB,
                    100
                ));

                // then:
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(200));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(300));

                // given:
                assert_ok!(Pallet::<Runtime>::mint_unsafe(
                    RuntimeOrigin::signed(ALICE),
                    CHARLIE,
                    100
                ));

                // then:
                assert_eq!(Balances::<Runtime>::get(&CHARLIE), Some(100));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(400));
            });
        }

        #[test]
        fn transfer_works() {
            StateBuilder::default().build_and_execute(|| {
                // given the initial state, when:
                assert_ok!(Pallet::<Runtime>::transfer(
                    RuntimeOrigin::signed(ALICE),
                    BOB,
                    50
                ));

                // then:
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(50));
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(150));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(200));

                // when:
                assert_ok!(Pallet::<Runtime>::transfer(
                    RuntimeOrigin::signed(BOB),
                    ALICE,
                    50
                ));

                // then:
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(100));
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(100));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(200));
            });
        }

        #[test]
        fn transfer_from_non_existent_fails() {
            StateBuilder::default().build_and_execute(|| {
                // given the initial state, when:
                assert_err!(
                    Pallet::<Runtime>::transfer(RuntimeOrigin::signed(CHARLIE), ALICE, 10),
                    "NonExistentAccount"
                );

                // then nothing has changed.
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(100));
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(100));
                assert_eq!(Balances::<Runtime>::get(&CHARLIE), None);
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(200));
            });
        }
    }
}

#[frame::pallet(dev_mode)]
pub mod pallet_v2 {
    use super::pallet::Balance;
    use frame::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type of the runtime.
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>
            + TryInto<Event<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Balances<T: Config> = StorageMap<_, _, T::AccountId, Balance>;

    #[pallet::storage]
    pub type TotalIssuance<T: Config> = StorageValue<_, Balance>;

    #[pallet::error]
    pub enum Error<T> {
        /// Account does not exist.
        NonExistentAccount,
        /// Account does not have enough balance.
        InsufficientBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A transfer succeeded.
        Transferred {
            from: T::AccountId,
            to: T::AccountId,
            amount: Balance,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub fn transfer(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // ensure sender has enough balance, and if so, calculate what is left after `amount`.
            let sender_balance =
                Balances::<T>::get(&sender).ok_or(Error::<T>::NonExistentAccount)?;
            let reminder = sender_balance
                .checked_sub(amount)
                .ok_or(Error::<T>::InsufficientBalance)?;

            Balances::<T>::mutate(&dest, |b| *b = Some(b.unwrap_or(0) + amount));
            Balances::<T>::insert(&sender, reminder);

            Self::deposit_event(Event::<T>::Transferred {
                from: sender,
                to: dest,
                amount,
            });

            Ok(())
        }
    }

    #[cfg(any(test, doc))]
    pub mod tests {
        use super::{super::pallet::tests::StateBuilder, *};
        use frame::testing_prelude::*;
        const ALICE: u64 = 1;
        const BOB: u64 = 2;

        pub mod runtime_v2 {
            use super::*;
            use crate::guides::your_first_pallet::pallet_v2 as pallet_currency;

            construct_runtime!(
                pub enum Runtime {
                    System: frame_system,
                    Currency: pallet_currency,
                }
            );

            #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
            impl frame_system::Config for Runtime {
                type Block = MockBlock<Runtime>;
                type AccountId = u64;
            }

            impl pallet_currency::Config for Runtime {
                type RuntimeEvent = RuntimeEvent;
            }
        }

        pub(crate) use runtime_v2::*;

        #[test]
        fn transfer_works() {
            StateBuilder::default().build_and_execute(|| {
                // skip the genesis block, as events are not deposited there and we need them for
                // the final assertion.
                System::set_block_number(ALICE);

                // given the initial state, when:
                assert_ok!(Pallet::<Runtime>::transfer(
                    RuntimeOrigin::signed(ALICE),
                    BOB,
                    50
                ));

                // then:
                assert_eq!(Balances::<Runtime>::get(&ALICE), Some(50));
                assert_eq!(Balances::<Runtime>::get(&BOB), Some(150));
                assert_eq!(TotalIssuance::<Runtime>::get(), Some(200));

                // now we can also check that an event has been deposited:
                assert_eq!(
                    System::read_events_for_pallet::<Event<Runtime>>(),
                    vec![Event::Transferred {
                        from: ALICE,
                        to: BOB,
                        amount: 50
                    }]
                );
            });
        }
    }
}
