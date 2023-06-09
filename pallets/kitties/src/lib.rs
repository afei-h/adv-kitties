#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use frame_support::{
		traits::{Currency, ExistenceRequirement, Randomness},
		PalletId,
	};
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::AccountIdConversion;
	use crate::migrations;

	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	//pub struct Kitty(pub [u8; 16]);
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 4],
	}
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId },
		KittyBought { who: T::AccountId, kitty_id: KittyId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOnSale,
		AlreadyOwned,
		NotOnSale,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::v1::migrate::<T>()
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create(origin: OriginFor<T>, name: [u8; 4]) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			let price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			// get new kitty's id
			let kitty_id = Self::get_next_id()?;
			// create a new kitty
			let dna = Self::random_value(&who);
			let kitty = Kitty { dna, name };

			// update storage
			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			// Emit an event.
			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
			name: [u8; 4],
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			let price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			// acquire parents' data
			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
			// Check that parents must be two different kitty
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			// Check both kitty_id_! and kitty_id_2 are valid
			// already checked when acquiring parents' data
			//ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			//ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			// get new kitty's id
			let kitty_id = Self::get_next_id()?;

			// generate child kitty's data and create child kitty
			let selector = Self::random_value(&who);
			let mut dna = [0u8; 16];
			for i in 0..selector.len() {
				// 0 choose kitty2 and 1 choose kitty1
				dna[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & !selector[i]);
			}
			let kitty = Kitty { dna, name };

			// update storage
			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			// Emit an event.
			Self::deposit_event(Event::KittyBred { who, kitty_id, kitty });

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check that the extrinsic was signed by kitty's owner.
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(who == owner, Error::<T>::NotOwner);

			// update storage
			KittyOwner::<T>::insert(kitty_id, &recipient);

			// Emit an event.
			Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Check that the kitty is exist
			Self::kitty_owner(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			// update storage
			KittyOnSale::<T>::insert(kitty_id, ());

			// Emit an event.
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// get kitty owner
			let owner = Self::kitty_owner(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

			// update storage
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);

			// Emit an event.
			Self::deposit_event(Event::KittyBought { who, kitty_id });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
