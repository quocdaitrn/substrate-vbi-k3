#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::inherent::Vec;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	// Kitty struct is used to store the kitty's data.
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T:Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}

	// Enum Gender
	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender{
		fn default()-> Self{
			Gender::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn number_of_kitties)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type NumOfKitties<T> = StorageValue<_, u32, ValueQuery>;

	// Key: dna
	// Value: kitty
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	// Key: AccountId
	// Value: Array of kitty DNAs
	#[pallet::storage]
	#[pallet::getter(fn kitties_by_owner)]
	pub(super) type UserKitties<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Kitty created.
		// Parameter: [dna, who, price].
		KittyStored(Vec<u8>, u32),

		// Kitty changed owner.
		// Parameter: [dna, old_owner, new_owner].
		KittyChangedOwner(Vec<u8>, T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		KittyNotExist,
		KittyAlreadyExist,
		KittyNotOwned,
		KittyNotConfiguredPrice,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let kitty = <Kitties<T>>::get(dna.clone());
			ensure!(kitty.is_none(), Error::<T>::KittyAlreadyExist);

			ensure!(price > 0, Error::<T>::KittyNotConfiguredPrice);

			let owner = who.clone();

			let gender = Self::gen_gender(dna.clone())?;
			let kitty = Kitty {
				dna: dna.clone(),
				gender: gender,
				price: price,
				owner: owner,
			};

			// Update storage.
			<Kitties<T>>::insert(dna.clone(), kitty);
			
			let mut current_number_kitties = <NumOfKitties<T>>::get();
			current_number_kitties += 1;
			NumOfKitties::<T>::put(current_number_kitties);

			let current_user_kitties = <UserKitties<T>>::get(&who);
			match current_user_kitties {
				Some(mut kitties) => {
					kitties.push(dna.clone());
					<UserKitties<T>>::insert(&who, kitties);
				},
				None => {
					let mut kitties = Vec::new();
					kitties.push(dna.clone());
					<UserKitties<T>>::insert(&who, kitties);
				},
			};

			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, price));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_kitty_owner(origin: OriginFor<T>, dna: Vec<u8>, new_owner: T::AccountId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			let owner = who.clone();

			let kitty_opt = <Kitties<T>>::get(dna.clone());
			ensure!(kitty_opt.is_some(), Error::<T>::KittyNotExist);

			let mut kitty = kitty_opt.unwrap();
			ensure!(kitty.owner == owner, Error::<T>::KittyNotOwned);

			let current_owner_kitties = <UserKitties<T>>::get(&who);
			match current_owner_kitties {
				Some(mut kitties) => {
					let index = kitties.iter().position(|x| x == &dna).unwrap();
					kitties.remove(index);
					<UserKitties<T>>::insert(&who, kitties);
				},
				None => {
					Err(Error::<T>::KittyNotOwned)?;
				},
			};

			let current_new_owner_kitties = <UserKitties<T>>::get(&new_owner);
			match current_new_owner_kitties {
				Some(mut kitties) => {
					kitties.push(dna.clone());
					<UserKitties<T>>::insert(&new_owner, kitties);
				},
				None => {
					let mut kitties = Vec::new();
					kitties.push(dna.clone());
					<UserKitties<T>>::insert(&new_owner, kitties);
				},
			};

			kitty.owner = new_owner.clone();
			<Kitties<T>>::insert(dna.clone(), kitty);

			// Emit an event.
			Self::deposit_event(Event::KittyChangedOwner(dna, owner, new_owner));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper functions
impl<T> Pallet<T> {
	fn gen_gender(dna: Vec<u8>) -> Result<Gender, Error<T>>{
		let mut res = Gender::Female;
		if dna.len() % 2 ==0 {
			res = Gender::Male;
		}
		Ok(res)
	}
}
