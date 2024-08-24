//! A shell pallet built with [`frame`].
//!
//! To get started with this pallet, try implementing the guide in
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>

// All pallets must be configured for `no_std`.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        // type WeightInfo;
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a claim has been created.
        ClaimCreated { who: T::AccountId, claim: T::Hash },
        /// Event emitted when a claim is revoked by the owner.
        ClaimRevoked { who: T::AccountId, claim: T::Hash },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The claim already exists.
        AlreadyClaimed,
        /// The claim does not exist, so it cannot be revoked.
        NoSuchClaim,
        /// The claim is owned by another account, so caller can't revoke it.
        NotClaimOwner,
    }

    #[pallet::storage]
    pub type Claims<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, BlockNumberFor<T>)>;

    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(Weight::default())]
        #[pallet::call_index(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let sender = ensure_signed(origin)?;

            ensure!(
                !Claims::<T>::contains_key(&claim),
                Error::<T>::AlreadyClaimed
            );

            let current_block_num = <frame_system::Pallet<T>>::block_number();

            Claims::<T>::insert(&claim, (&sender, current_block_num));

            Self::deposit_event(Event::ClaimCreated { who: sender, claim });

            Ok(())
        }

        #[pallet::weight(Weight::default())]
        #[pallet::call_index(1)]
        pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let sender = ensure_signed(origin)?;

            let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            Claims::<T>::remove(&claim);

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimRevoked { who: sender, claim });

            Ok(())
        }
    }
}

pub mod weights {
    // Placeholder struct for the pallet weights
    pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
}
