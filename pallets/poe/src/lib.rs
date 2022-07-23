#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// define config
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxBytesInHash: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// define storage
	#[pallet::storage]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxBytesInHash>,
		(T::AccountId, T::BlockNumber),
		OptionQuery,
	>;

	// define event callback
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		// when claim creaated, account, claim
		ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		// when claim revoked, account, claim
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		// when claim transmit, from account, to account, claim
		ClaimTransmit(T::AccountId, T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
	}

    // define error
	#[pallet::error]
	pub enum Error<T> {
        // proof is already claimed
		ProofAlreadyClaimed,
        // Proof not exist
		NoSuchProof,
        // Proof owner not match
		NotProofOwner,
	}

    // define calling methods
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)] // weight used to avoid being attack

        // create claim to `origin` account
		pub fn create_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>,
		) -> DispatchResult {

            // ensure cheking
            // https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

            // check if the proof was defined
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

            // get block number from Frame System Pallet
			let current_block = <frame_system::Pallet<T>>::block_number();

            // insert proof, sender and block number
			Proofs::<T>::insert(&proof, (&sender, current_block));

			// send a ClaimCreated event
			Self::deposit_event(Event::ClaimCreated(sender, proof));

			Ok(())
		}

        // revoke claim of `origin` account method
		#[pallet::weight(10_000)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>, //
		) -> DispatchResult {

            // ensure cheking
            // https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

            // check if the proof is existed
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // get the owner of proof
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

            // check if revoker has the proof
			ensure!(sender == owner, Error::<T>::NotProofOwner);

            // remove proof from block
			Proofs::<T>::remove(&proof);

            // send a ClaimRevoked event
			Self::deposit_event(Event::ClaimRevoked(sender, proof));
			Ok(())
		}

        // transmit claim from `origin` account to `receiver` account
		#[pallet::weight(100_000)]
		pub fn transmit_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>,
			receiver: T::AccountId,
		) -> DispatchResult {

            // ensure cheking
            // https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

            // check if the proof is existed
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // get the owner of proof
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

            // check if revoker has the proof
			ensure!(sender == owner, Error::<T>::NotProofOwner);

            // get block number from Frame System Pallet
			let current_block = <frame_system::Pallet<T>>::block_number();

            // transmit proof
            Proofs::<T>::mutate(&proof, |value| {
                value.as_mut().unwrap().0 = receiver.clone();
                value.as_mut().unwrap().1 = current_block;
            });

			Self::deposit_event(Event::ClaimTransmit(sender, receiver, proof));
			Ok(())
		}
	}
}
