//->v2

use frame_support::{
	pallet_prelude::*, storage::StoragePrefixedMap, traits::GetStorageVersion, weights::Weight,
};

use crate::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};
use frame_system::pallet_prelude::*;

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct V1Kitty {
	pub dna: [u8; 16],
	pub name: [u8; 8],
}
#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct V0Kitty(pub [u8; 16]);

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if current_version != 2 {
		return Weight::zero()
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	if on_chain_version > 1 {
		return Weight::zero()
	} else if on_chain_version == 0 {
		for (index, kitty) in
			storage_key_iter::<KittyId, V0Kitty, Blake2_128Concat>(module, item).drain()
		{
			let new_kitty = Kitty { dna: kitty.0, name: *b"abcdefgh" };

			Kitties::<T>::insert(index, &new_kitty);
		}
	} else if on_chain_version == 1 {
		for (index, kitty) in
			storage_key_iter::<KittyId, V1Kitty, Blake2_128Concat>(module, item).drain()
		{
			let new_kitty = Kitty { dna: kitty.dna, name: *b"abcd0000" };

			Kitties::<T>::insert(index, &new_kitty);
		}
	}

	Weight::zero()
}
