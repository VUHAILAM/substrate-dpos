#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod models;

// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/polkadot_sdk/frame_runtime/index.html
// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html
// https://paritytech.github.io/polkadot-sdk/master/frame_support/attr.pallet.html#dev-mode-palletdev_mode
#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::models::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible, FindAuthor},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	pub trait ReportNewValidatorSet<AccountId> {
		fn report_new_validator_set(_new_set: Vec<AccountId>) {}
	}

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// The maximum number of authorities that the pallet can hold.
		type MaxValidators: Get<u32>;

		/// The maximum number of authorities that the pallet can hold.
		#[pallet::constant]
		type MaxCandidates: Get<u32>;

		/// The maximum number of delegators that the candidate can have
		/// If the number of delegators reaches the maximum, delegator with the lowest amount
		/// will be replaced by the new delegator if the new delegation is higher
		#[pallet::constant]
		type MaxCandidateDelegators: Get<u32>;

		/// Find the author of a block. A fake provide for this type is provided in the runtime. You
		/// can use a similar mechanism in your tests.
		type FindAuthor: FindAuthor<Self::AccountId>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;
	}

	/// The pallet's storage items.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#storage
	/// https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/attr.storage.html
	#[pallet::storage]
	pub type CandidatePool<T: Config> = CountedStorageMap<_, Twox64Concat, T::AccountId, Candidate<T>, OptionQuery>;
	#[pallet::storage]
	pub type DelegateCountMap<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;
	#[pallet::storage]
	pub type DelegationInfos<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, Delegation<T>, OptionQuery>;
	/// Pallets use events to inform users when important changes are made.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// We usually use passive tense for events.
		SomethingStored { something: u32, who: T::AccountId },
		/// Event emitted when there is a new candidate registered
		CandidateRegistered { candidate_id: T::AccountId, initial_bond: BalanceOf<T> },
	}

	/// Errors inform users that something went wrong.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error
	#[pallet::error]
	pub enum Error<T> {
		TooManyValidators,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	/// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#dispatchables
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example of directly updating the authorities into [`Config::ReportNewValidatorSet`].
		pub fn force_report_new_validators(
			origin: OriginFor<T>,
			new_set: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				(new_set.len() as u32) < T::MaxValidators::get(),
				Error::<T>::TooManyValidators
			);
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
			Ok(())
		}

		pub fn register_as_candidate(
			origin: OriginFor<T>,
			initial_bond: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let candidate = Candidate::new(initial_bond);
			CandidatePool::<T>::insert(&who, candidate);
			Self::deposit_event(Event::CandidateRegistered { candidate_id: who, initial_bond });
			Ok(())
		}

		pub fn delegate(
			origin: OriginFor<T>,
			candidate_id: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut candidate = CandidatePool::<T>::get(&candidate_id).ok_or("Candidate not found")?;
			let mut delegator = Delegation::new(amount);
			DelegationInfos::<T>::insert(&candidate_id, &who, delegator);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// A function to get you an account id for the current block author.
		pub fn find_author() -> Option<T::AccountId> {
			// If you want to see a realistic example of the `FindAuthor` interface, see
			// `pallet-authorship`.
			T::FindAuthor::find_author::<'_, Vec<_>>(Default::default())
		}
	}
}