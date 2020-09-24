#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{Parameter, decl_module, decl_event, decl_storage, decl_error, ensure, dispatch};
use sp_runtime::traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup};
use frame_system::ensure_signed;
use sp_runtime::traits::One;
use pallet_balances as balances;

/// The module configuration trait.
pub trait Trait: frame_system::Trait + balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The units in which we record balances.
///	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	/// The arithmetic type of asset identifier.
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		/// Issue a new class of fungible assets. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `AssetId` instance: this will be specified in the `Issued` event.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 storage mutation (codec `O(1)`).
		/// - 2 storage writes (condec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn issue(origin, #[compact] total: T::Balance) {
			let origin = ensure_signed(origin)?;
			// save 0 for native currency
			let mut id = Self::next_asset_id();
			if id == Zero::zero() {
				id += One::one();
			}
			<NextAssetId<T>>::mutate(|id| {
                if *id == Zero::zero() {
                    *id += One::one();
                }
                *id += One::one();
            });

			<Balances<T>>::insert((id, &origin), total);
			<TotalSupply<T>>::insert(id, total);
			<Creator<T>>::insert(id, &origin);

			Self::deposit_event(RawEvent::Issued(id, origin, total));
		}

		/// Mint any assets of `id` owned by `origin`.
        ///
        /// # <weight>
        /// - `O(1)`
        /// - 1 storage mutation (codec `O(1)`).
        /// - 1 storage deletion (codec `O(1)`).
        /// - 1 event.
        /// # </weight>
        #[weight = 0]
        fn mint(origin,
             #[compact] id: T::AssetId,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: <T as balances::Trait>::Balance
        ){
            let origin = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;
            let creator = <Creator<T>>::get(id);
            ensure!(origin == creator, Error::<T>::NotTheCreator);
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            Self::deposit_event(RawEvent::Minted(id, target.clone(), amount));
            <Balances<T>>::mutate((id, target), |balance| *balance += amount);
        }


        /// Burn any assets of `id` owned by `origin`.
        ///
        /// # <weight>
        /// - `O(1)`
        /// - 1 storage mutation (codec `O(1)`).
        /// - 1 storage deletion (codec `O(1)`).
        /// - 1 event.
        /// # </weight>
        #[weight = 0]
        fn burn(origin,
            #[compact] id: T::AssetId,
           target: <T::Lookup as StaticLookup>::Source,
           #[compact] amount: <T as balances::Trait>::Balance
       ){
           let origin = ensure_signed(origin)?;
           let origin_account = (id, origin.clone());
           let origin_balance = <Balances<T>>::get(&origin_account);
           ensure!(!amount.is_zero(), Error::<T>::AmountZero);
           ensure!(origin_balance >= amount, Error::<T>::BalanceLow);

           Self::deposit_event(RawEvent::Burned(id, origin, amount));
           <Balances<T>>::insert(origin_account, origin_balance - amount);
       }

		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn transfer(origin,
			#[compact] id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;
			let origin_account = (id, origin.clone());
			let origin_balance = <Balances<T>>::get(&origin_account);
			let target = T::Lookup::lookup(target)?;
			ensure!(!amount.is_zero(), Error::<T>::AmountZero);
			ensure!(origin_balance >= amount, Error::<T>::BalanceLow);

			Self::deposit_event(RawEvent::Transferred(id, origin, target.clone(), amount));
			<Balances<T>>::insert(origin_account, origin_balance - amount);
			<Balances<T>>::mutate((id, target), |balance| *balance += amount);
		}

		/// Destroy any assets of `id` owned by `origin`.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 storage mutation (codec `O(1)`).
		/// - 1 storage deletion (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn destroy(origin, #[compact] id: T::AssetId) {
			let origin = ensure_signed(origin)?;
			let balance = <Balances<T>>::take((id, &origin));
			ensure!(!balance.is_zero(), Error::<T>::BalanceZero);

			<TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
			Self::deposit_event(RawEvent::Destroyed(id, origin, balance));
		}
	}
}