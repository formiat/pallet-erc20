#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{ArithmeticError, DispatchError};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type BalanceOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn is_init)]
	pub type Init<T: Config> = StorageValue<_, bool>;

	#[pallet::storage]
	#[pallet::getter(fn allowance)]
	pub(super) type Allowance<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token was initialized by user
		Initialized(T::AccountId),
		/// Tokens successfully transferred between users
		Transfer(T::AccountId, T::AccountId, u64),
		/// Allowance successfully created
		Approval(T::AccountId, T::AccountId, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Attempted to initialize the token after it had already been initialized.
		AlreadyInitialized,
		/// Attempted to transfer more funds than were available
		InsufficientFunds,
		/// Attempted to transfer more funds than approved
		InsufficientApprovedFunds,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn init(origin: OriginFor<T>, total_supply: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_init().is_none(), <Error<T>>::AlreadyInitialized);

			<TotalSupply<T>>::put(total_supply);
			<BalanceOf<T>>::insert(&sender, total_supply);

			<Init<T>>::put(true);

			Self::deposit_event(Event::Initialized(sender));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, value: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// get the balance values
			let from_balance = Self::balance_of(&sender).unwrap();
			// TODO: Check if `unwrap_or(0)` is right
			let to_balance = Self::balance_of(&to).unwrap_or(0);

			// Calculate new balances
			let updated_from_balance =
				from_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = to_balance
				.checked_add(value)
				.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

			// Write new balances to storage
			<BalanceOf<T>>::insert(&sender, updated_from_balance);
			<BalanceOf<T>>::insert(&to, updated_to_balance);

			Self::deposit_event(Event::Transfer(sender, to, value));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve(origin: OriginFor<T>, spender: T::AccountId, value: u64) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			<Allowance<T>>::insert((owner.clone(), spender.clone()), value);

			Self::deposit_event(Event::Approval(owner, spender, value));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			owner: T::AccountId,
			to: T::AccountId,
			value: u64,
		) -> DispatchResult {
			let spender = ensure_signed(origin)?;

			// get the balance values
			// TODO: Check if `unwrap_or(0)` is right
			let owner_balance = Self::balance_of(&owner).unwrap_or(0);
			// TODO: Check if `unwrap_or(0)` is right
			let to_balance = Self::balance_of(&to).unwrap_or(0);

			// get the allowance value
			let approved_balance = Self::allowance((owner.clone(), spender.clone())).unwrap_or(0);

			// Calculate new balances
			let updated_approved_balance = approved_balance
				.checked_sub(value)
				.ok_or(<Error<T>>::InsufficientApprovedFunds)?;
			let updated_owner_balance =
				owner_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = to_balance
				.checked_add(value)
				.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

			// Write new balances to storage
			<BalanceOf<T>>::insert(owner.clone(), updated_owner_balance);
			<BalanceOf<T>>::insert(&to, updated_to_balance);

			// Write new allowance to storage
			<Allowance<T>>::insert((owner.clone(), spender.clone()), updated_approved_balance);

			Self::deposit_event(Event::Transfer(owner, to, value));

			Ok(())
		}
	}
}
