#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
    decl_module, decl_storage, decl_error, decl_event, ensure, StorageMap, Parameter,
    traits::{Randomness, Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
};
use sp_io::hashing::blake2_128;
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    DispatchError,
    traits::{AtLeast32Bit, Bounded, Member},
};
use crate::linked_item::{LinkedList, LinkedItem};

mod linked_item;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Randomness: Randomness<Self::Hash>;

    type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;

    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnedKittiesList<T> = LinkedList<OwnedKitties<T>, <T as system::Trait>::AccountId, <T as Trait>::KittyIndex>;
type KittyChildrenList<T> = LinkedList<KittyChildren<T>, <T as Trait>::KittyIndex, <T as Trait>::KittyIndex>;

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<(Kitty, BalanceOf<T>)>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;

        pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

        pub KittyParents get(fn kitty_parents): map hasher(blake2_128_concat) T::KittyIndex => Option<(T::KittyIndex, T::KittyIndex)>;
        pub KittyChildren get(fn kitty_children): map hasher(blake2_128_concat) (T::KittyIndex, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
    }
}

decl_error! {
	pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
        NotKittyOwner,
	}
}

decl_event!(
    pub enum Event<T> where 
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        <T as Trait>::KittyIndex,
    {
        Created(AccountId, KittyIndex, Balance),
        Transferred(AccountId, AccountId, KittyIndex, Balance),
    }
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin, amount: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);

            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty, amount);
            
            T::Currency::reserve(&sender, amount)
					.map_err(|_| "locker can't afford to lock the amount requested")?;

            Self::deposit_event(RawEvent::Created(sender, kitty_id, amount));
        }

        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::NotKittyOwner);

            let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            let (_, amount) = kitty;

            T::Currency::unreserve(&sender, amount);
            T::Currency::transfer(&sender, &to, amount, AllowDeath)?;
            T::Currency::reserve(&to, amount)
					.map_err(|_| "locker can't afford to lock the amount requested")?;

            <OwnedKittiesList<T>>::remove(&sender, kitty_id);
		    Self::insert_owned_kitty(&to, kitty_id);
            
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id, amount));
        }

        #[weight = 0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex, amount: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2, amount)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id, amount));
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
    }
    
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }
    
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty, amount: BalanceOf<T>) {
        Kitties::<T>::insert(kitty_id, (kitty, amount));
        KittiesCount::<T>::put(kitty_id + 1.into());
        Self::insert_owned_kitty(owner, kitty_id);
    }

    fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		<OwnedKittiesList<T>>::append(owner, kitty_id);
		<KittyOwners<T>>::insert(kitty_id, owner);
	}
    
    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex, amount: BalanceOf<T>) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_1))), Error::<T>::NotKittyOwner);
		ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_2))), Error::<T>::NotKittyOwner);
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        T::Currency::reserve(sender, amount)
					.map_err(|_| "locker can't afford to lock the amount requested")?;

        let kitty_id = Self::next_kitty_id()?;

        let (kitty1_, _) = kitty1;
        let (kitty2_, _) = kitty2;
        let kitty1_dna = kitty1_.0;
        let kitty2_dna = kitty2_.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }

        <KittyChildrenList<T>>::append(&kitty_id_1, kitty_id);
        <KittyChildrenList<T>>::append(&kitty_id_2, kitty_id);
		<KittyParents<T>>::insert(kitty_id, (kitty_id_1, kitty_id_2));
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna), amount);
        Ok(kitty_id)
    }
}

