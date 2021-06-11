// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

mod cid;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use sp_runtime::{
	traits::{Hash, StaticLookup, Zero},
	RuntimeDebug,
};
use sp_std::prelude::*;

use frame_support::{
	codec::{Decode, Encode},
	dispatch::{DispatchError, DispatchResult, Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::{
		ChangeMembers, Currency, InitializeMembers, IsSubType, LockableCurrency, OnUnbalanced,
		ReservableCurrency,
	},
	weights::{Pays, Weight},
};
pub use pallet::*;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

type Url = Vec<u8>;

type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type NegativeImbalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

const IDENTITY_FIELD_DISPLAY: u64 =
	0b0000000000000000000000000000000000000000000000000000000000000001;
const IDENTITY_FIELD_WEB: u64 = 0b0000000000000000000000000000000000000000000000000000000000000100;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MemberRole {
	Founder,
	Fellow,
	Ally,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum UserIdentity<AccountId> {
	AccountId(AccountId),
	Website(Url),
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// The outer call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ IsSubType<Call<Self, I>>
			+ IsType<<Self as frame_system::Config>::Call>;

		/// Origin from which the next tabled referendum may be forced; this allows for the tabling of
		/// a majority-carries referendum.
		type SuperMajorityOrigin: EnsureOrigin<Self::Origin>;

		/// The currency used for deposits.
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;

		/// What to do with slashed funds.
		type Slashed: OnUnbalanced<NegativeImbalanceOf<Self, I>>;

		/// What to do with genesis members
		type InitializeMembers: InitializeMembers<Self::AccountId>;

		/// The receiver of the signal for when the members have changed.
		type MembershipChanged: ChangeMembers<Self::AccountId>;

		type IdentityVerifier: IdentityVerifier<Self::AccountId>;

		type ProposalProvider: ProposalProvider<Self::AccountId, Self::Hash, Self::Proposal>;

		/// The minimum amount of a deposit required for submit candidacy.
		#[pallet::constant]
		type CandidateDeposit: Get<BalanceOf<Self, I>>;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		FoundersAlreadyInitialized,
		AlreadyCandidate,
		NotCandidate,
		AlreadyMember,
		NotMember,
		NotAlly,
		NotFounder,
		NotVotableMember,
		AlreadyElevated,
		AlreadyInBlacklist,
		NotInBlacklist,
		KickingMember,
		InsufficientCandidateFunds,
		NoIdentity,
		MissingProposalHash,
		NotVetoableProposal,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		NewRule(cid::Cid),
		NewAnnouncement(cid::Cid),
		FoundersInitialized(Vec<T::AccountId>),
		CandidateAdded(T::AccountId, Option<T::AccountId>, Option<BalanceOf<T, I>>),
		CandidateApproved(T::AccountId),
		CandidateRejected(T::AccountId),
		AllyElevated(T::AccountId),
		MemberRetired(T::AccountId),
		MemberKicked(T::AccountId),
		BlacklistAdded(Vec<UserIdentity<T::AccountId>>),
		BlacklistRemoved(Vec<UserIdentity<T::AccountId>>),
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub founders: Vec<T::AccountId>,
		pub fellows: Vec<T::AccountId>,
		pub allies: Vec<T::AccountId>,
		pub phantom: PhantomData<(T, I)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				founders: Vec::new(),
				fellows: Vec::new(),
				allies: Vec::new(),
				phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			for m in self
				.founders
				.iter()
				.chain(self.fellows.iter())
				.chain(self.allies.iter())
			{
				assert!(
					Pallet::<T, I>::has_identity(m),
					"Member does not set identity!"
				);
			}

			if !self.founders.is_empty() {
				assert!(
					!Pallet::<T, I>::has_member(MemberRole::Founder),
					"Founders are already initialized!"
				);
				Members::<T, I>::insert(MemberRole::Founder, self.founders.clone());
			}
			if !self.fellows.is_empty() {
				assert!(
					!Pallet::<T, I>::has_member(MemberRole::Fellow),
					"Fellows are already initialized!"
				);
				Members::<T, I>::insert(MemberRole::Fellow, self.fellows.clone());
			}
			if !self.allies.is_empty() {
				Members::<T, I>::insert(MemberRole::Ally, self.allies.clone())
			}

			//T::InitializeMembers::initialize_members(
			//	&[self.founders.as_slice(), self.fellows.as_slice()].concat(),
			//)
		}
	}

	/// A ipfs cid of the rules of this alliance concerning membership.
	/// Any member can propose rules, other members make a traditional majority-wins
	/// vote to determine if the rules take effect.
	/// The founder has a special one-vote veto right to the rules setting.
	#[pallet::storage]
	#[pallet::getter(fn rule)]
	pub type Rule<T: Config<I>, I: 'static = ()> = StorageValue<_, cid::Cid, OptionQuery>;

	/// Maps proposal hash and identity info.
	#[pallet::storage]
	#[pallet::getter(fn announcements)]
	pub type Announcements<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<cid::Cid>, ValueQuery>;

	/// The member who have locked a candidate deposit.
	#[pallet::storage]
	#[pallet::getter(fn deposit_of)]
	pub type DepositOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T, I>, OptionQuery>;

	/// The current set of candidates; outsiders who are attempting to become members.
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// The current set of alliance members(founder/fellow/ally).
	///
	/// Note: ally can’t proposal or vote on motions, only for show.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, MemberRole, Vec<T::AccountId>, ValueQuery>;

	/// The set of kicking members.
	#[pallet::storage]
	#[pallet::getter(fn kicking_member)]
	pub type KickingMembers<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	/// Maps proposal hash and identity info.
	#[pallet::storage]
	#[pallet::getter(fn account_blacklist)]
	pub type AccountBlacklist<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn website_blacklist)]
	pub type WebsiteBlacklist<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<Url>, ValueQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Add a new proposal to be voted on.
		///
		/// Requires the sender to be elevated member(founders/fellows).
		#[pallet::weight(0)]
		pub(super) fn propose(
			origin: OriginFor<T>,
			proposal: Box<<T as Config<I>>::Proposal>,
		) -> DispatchResult {
			let proposor = ensure_signed(origin)?;
			ensure!(
				Self::is_votable_member(&proposor),
				Error::<T, I>::NotVotableMember
			);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			if let Some(Call::kick_member(ref strike)) = proposal.is_sub_type() {
				let strike = T::Lookup::lookup(strike.clone())?;
				<KickingMembers<T, I>>::insert(strike, true);
			}

			let threshold = 2 * Self::votable_member_count() / 3 + 1;
			T::ProposalProvider::propose_proposal(proposor, threshold, *proposal, proposal_hash)?;
			Ok(())
		}

		/// Disapprove a proposal, close, and remove it from the system, regardless of its current state.
		///
		/// Must be called by the founders.
		#[pallet::weight(0)]
		pub(super) fn veto(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResult {
			let proposor = ensure_signed(origin)?;
			ensure!(Self::is_founder(&proposor), Error::<T, I>::NotFounder);

			let proposal = T::ProposalProvider::proposal_of(proposal_hash);
			ensure!(proposal.is_some(), Error::<T, I>::MissingProposalHash);
			match proposal.expect("proposal must be exist; qed").is_sub_type() {
				Some(Call::set_rule(..)) | Some(Call::elevate_ally(..)) => {
					T::ProposalProvider::veto_proposal(proposal_hash);
					Ok(())
				}
				_ => Err(Error::<T, I>::NotVetoableProposal.into()),
			}
		}

		#[pallet::weight(0)]
		pub(super) fn close(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
			index: ProposalIndex,
			proposal_weight_bound: Weight,
			length_bound: u32,
		) -> DispatchResult {
			let proposor = ensure_signed(origin)?;
			ensure!(
				Self::is_votable_member(&proposor),
				Error::<T, I>::NotVotableMember
			);

			let proposal = T::ProposalProvider::proposal_of(proposal_hash);
			ensure!(proposal.is_some(), Error::<T, I>::MissingProposalHash);

			let (_, pays) = T::ProposalProvider::close_proposal(
				proposal_hash,
				index,
				proposal_weight_bound,
				length_bound,
			)?;
			if Pays::No == pays {
				if let Some(Call::kick_member(ref strike)) =
					proposal.expect("proposal must be exist; qed").is_sub_type()
				{
					let strike = T::Lookup::lookup(strike.clone())?;
					<KickingMembers<T, I>>::remove(strike);
				}
			}
			Ok(())
		}

		/// Initialize the founders to the given members.
		#[pallet::weight(0)]
		pub(super) fn init_founders(
			origin: OriginFor<T>,
			founders: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				!Self::has_member(MemberRole::Founder),
				Error::<T, I>::FoundersAlreadyInitialized
			);
			for founder in &founders {
				ensure!(Self::has_identity(founder), Error::<T, I>::NoIdentity);
			}

			let mut founders = founders;
			founders.sort();
			T::InitializeMembers::initialize_members(&founders);
			Members::<T, I>::insert(&MemberRole::Founder, founders.clone());

			Self::deposit_event(Event::FoundersInitialized(founders));
			Ok(())
		}

		/// A IPFS cid of the rules of this alliance concerning membership.
		#[pallet::weight(0)]
		pub(super) fn set_rule(origin: OriginFor<T>, rule: cid::Cid) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;

			Rule::<T, I>::put(&rule);

			Self::deposit_event(Event::NewRule(rule));
			Ok(())
		}

		/// Announcement IPFS Hash about dispute between two allies and other issues.
		/// Proposer should publish in polkassembly.io first and talked with others,
		/// then publish the post into IPFS. Create a ID.
		#[pallet::weight(0)]
		pub(super) fn announce(origin: OriginFor<T>, announcement: cid::Cid) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;

			<Announcements<T, I>>::get().push(announcement);

			Self::deposit_event(Event::NewAnnouncement(announcement));
			Ok(())
		}

		/// Submit oneself for candidacy.
		///
		/// Account must have enough transferable funds in it to pay the candidate deposit.
		#[pallet::weight(0)]
		pub(super) fn submit_candidacy(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				!Self::is_account_blacklist(&who),
				Error::<T, I>::AlreadyInBlacklist
			);
			ensure!(!Self::is_candidate(&who), Error::<T, I>::AlreadyCandidate);
			ensure!(!Self::is_member(&who), Error::<T, I>::AlreadyMember);
			// check user self or parent should has verified identity to reuse display name and website.
			ensure!(Self::has_identity(&who), Error::<T, I>::NoIdentity);

			let deposit = T::CandidateDeposit::get();
			T::Currency::reserve(&who, deposit)
				.map_err(|_| Error::<T, I>::InsufficientCandidateFunds)?;
			<DepositOf<T, I>>::insert(&who, deposit);

			Self::add_candidate(&who)?;

			Self::deposit_event(Event::CandidateAdded(who, None, Some(deposit)));
			Ok(())
		}

		/// As a elevated member, nominate for someone to join alliance.
		///
		/// There is no deposit required to the nominees.
		///
		/// The dispatch origin for this call must be _Signed_ and a elevated member.
		#[pallet::weight(0)]
		pub(super) fn nominate_candidacy(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let nominator = ensure_signed(origin)?;
			ensure!(
				Self::is_votable_member(&nominator),
				Error::<T, I>::NotVotableMember
			);
			let who = T::Lookup::lookup(who)?;
			ensure!(
				!Self::is_account_blacklist(&who),
				Error::<T, I>::AlreadyInBlacklist
			);
			ensure!(!Self::is_candidate(&who), Error::<T, I>::AlreadyCandidate);
			ensure!(!Self::is_member(&who), Error::<T, I>::AlreadyMember);
			// check user self or parent should has verified identity to reuse display name and website.
			ensure!(Self::has_identity(&who), Error::<T, I>::NoIdentity);

			Self::add_candidate(&who)?;

			Self::deposit_event(Event::CandidateAdded(who, Some(nominator), None));
			Ok(())
		}

		/// Approve a `Candidate` to be a `Ally`.
		#[pallet::weight(0)]
		pub(super) fn approve_candidate(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let candidate = T::Lookup::lookup(candidate)?;
			ensure!(Self::is_candidate(&candidate), Error::<T, I>::NotCandidate);
			ensure!(!Self::is_member(&candidate), Error::<T, I>::AlreadyMember);

			Self::remove_candidate(&candidate)?;
			Self::add_member(&candidate, MemberRole::Ally)?;

			Self::deposit_event(Event::CandidateApproved(candidate));
			Ok(())
		}

		/// Reject a `Candidate` to be a `Outsider`.
		/// Only the members (`Fellows` and `Founders`) can vote to approve/reject the `Candidate`.
		#[pallet::weight(0)]
		pub(super) fn reject_candidate(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let candidate = T::Lookup::lookup(candidate)?;
			ensure!(Self::is_candidate(&candidate), Error::<T, I>::NotCandidate);
			ensure!(!Self::is_member(&candidate), Error::<T, I>::AlreadyMember);

			Self::remove_candidate(&candidate)?;
			if let Some(deposit) = DepositOf::<T, I>::take(&candidate) {
				T::Slashed::on_unbalanced(T::Currency::slash_reserved(&candidate, deposit).0);
			}

			Self::deposit_event(Event::CandidateRejected(candidate));
			Ok(())
		}

		/// As a active member, elevate a ally to fellow.
		///
		/// The dispatch origin for this call must be _Signed_ and a member.
		#[pallet::weight(0)]
		pub(super) fn elevate_ally(
			origin: OriginFor<T>,
			ally: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let ally = T::Lookup::lookup(ally)?;
			ensure!(Self::is_ally(&ally), Error::<T, I>::NotAlly);
			ensure!(
				!Self::is_votable_member(&ally),
				Error::<T, I>::AlreadyElevated
			);

			Self::remove_member(&ally, MemberRole::Ally)?;
			Self::add_member(&ally, MemberRole::Fellow)?;

			Self::deposit_event(Event::AllyElevated(ally));
			Ok(())
		}

		/// As a member, back to outsider and unlock deposit.
		#[pallet::weight(0)]
		pub(super) fn retire(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::is_kicking(&who), Error::<T, I>::KickingMember);

			if let Some(role) = Self::member_role_of(&who) {
				Self::remove_member(&who, role)?;
				if let Some(deposit) = DepositOf::<T, I>::take(&who) {
					let err_amount = T::Currency::unreserve(&who, deposit);
					debug_assert!(err_amount.is_zero());
				}
				Self::deposit_event(Event::MemberRetired(who));
				Ok(())
			} else {
				Err(Error::<T, I>::NotMember.into())
			}
		}

		/// Kick a member to outsider with its deposit slashed.
		#[pallet::weight(0)]
		pub(super) fn kick_member(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let member = T::Lookup::lookup(who)?;
			ensure!(!Self::is_kicking(&member), Error::<T, I>::KickingMember);

			if let Some(role) = Self::member_role_of(&member) {
				Self::remove_member(&member, role)?;
				if let Some(deposit) = DepositOf::<T, I>::take(member.clone()) {
					T::Slashed::on_unbalanced(T::Currency::slash_reserved(&member, deposit).0);
				}
				Self::deposit_event(Event::MemberKicked(member));
				Ok(())
			} else {
				Err(Error::<T, I>::NotMember.into())
			}
		}

		/// Add websites or addresses into blacklist.
		#[pallet::weight(0)]
		pub(super) fn add_blacklist(
			origin: OriginFor<T>,
			infos: Vec<UserIdentity<T::AccountId>>,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let mut accounts = vec![];
			let mut webs = vec![];
			for info in infos.iter() {
				ensure!(!Self::is_blacklist(info), Error::<T, I>::AlreadyInBlacklist);
				match info {
					UserIdentity::AccountId(who) => accounts.push(who.clone()),
					UserIdentity::Website(url) => webs.push(url.clone()),
				}
			}
			Self::do_add_blacklist(&mut accounts, &mut webs)?;
			Self::deposit_event(Event::BlacklistAdded(infos));
			Ok(())
		}

		/// Remove websites or addresses form blacklist.
		#[pallet::weight(0)]
		pub(super) fn remove_blacklist(
			origin: OriginFor<T>,
			infos: Vec<UserIdentity<T::AccountId>>,
		) -> DispatchResult {
			T::SuperMajorityOrigin::ensure_origin(origin)?;
			let mut accounts = vec![];
			let mut webs = vec![];
			for info in infos.iter() {
				ensure!(Self::is_blacklist(info), Error::<T, I>::NotInBlacklist);
				match info {
					UserIdentity::AccountId(who) => accounts.push(who.clone()),
					UserIdentity::Website(url) => webs.push(url.clone()),
				}
			}
			Self::do_remove_blacklist(&mut accounts, &mut webs)?;
			Self::deposit_event(Event::BlacklistRemoved(infos));
			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Check if a user is a candidate.
	fn is_candidate(who: &T::AccountId) -> bool {
		<Candidates<T, I>>::get().contains(who)
	}

	/// Add a candidate to the sorted candidate list.
	fn add_candidate(who: &T::AccountId) -> DispatchResult {
		let mut candidates = <Candidates<T, I>>::get();
		let pos = candidates
			.binary_search(who)
			.err()
			.ok_or(Error::<T, I>::AlreadyCandidate)?;
		candidates.insert(pos, who.clone());
		Candidates::<T, I>::put(candidates);
		Ok(())
	}

	/// Remove a candidate from the candidates list.
	fn remove_candidate(who: &T::AccountId) -> DispatchResult {
		let mut candidates = <Candidates<T, I>>::get();
		let pos = candidates
			.binary_search(who)
			.ok()
			.ok_or(Error::<T, I>::NotCandidate)?;
		candidates.remove(pos);
		Candidates::<T, I>::put(candidates);
		Ok(())
	}

	fn has_member(role: MemberRole) -> bool {
		!Members::<T, I>::get(role).is_empty()
	}

	fn member_role_of(who: &T::AccountId) -> Option<MemberRole> {
		Members::<T, I>::iter()
			.find_map(|(r, members)| if members.contains(who) { Some(r) } else { None })
	}

	/// Check if a user is a alliance member.
	fn is_member(who: &T::AccountId) -> bool {
		Self::member_role_of(who).is_some()
	}

	fn is_member_of(who: &T::AccountId, role: MemberRole) -> bool {
		Members::<T, I>::get(role).contains(&who)
	}

	fn is_founder(who: &T::AccountId) -> bool {
		Self::is_member_of(who, MemberRole::Founder)
	}

	fn is_fellow(who: &T::AccountId) -> bool {
		Self::is_member_of(who, MemberRole::Fellow)
	}

	fn is_ally(who: &T::AccountId) -> bool {
		Self::is_member_of(who, MemberRole::Ally)
	}

	fn is_votable_member(who: &T::AccountId) -> bool {
		Self::is_founder(who) || Self::is_fellow(who)
	}

	fn votable_member_count() -> u32 {
		let founders = Members::<T, I>::get(MemberRole::Founder);
		let fellows = Members::<T, I>::get(MemberRole::Fellow);
		(founders.len() + fellows.len()) as u32
	}

	fn votable_member_sorted() -> Vec<T::AccountId> {
		let mut founders = Members::<T, I>::get(MemberRole::Founder);
		let mut fellows = Members::<T, I>::get(MemberRole::Fellow);
		founders.append(&mut fellows);
		founders.sort();
		founders
	}

	fn is_kicking(who: &T::AccountId) -> bool {
		<KickingMembers<T, I>>::contains_key(&who)
	}

	/// Add a user to the sorted alliance member set.
	fn add_member(who: &T::AccountId, role: MemberRole) -> DispatchResult {
		<Members<T, I>>::mutate(role, |members| -> DispatchResult {
			let pos = members
				.binary_search(&who)
				.err()
				.ok_or(Error::<T, I>::AlreadyMember)?;
			members.insert(pos, who.clone());
			Ok(())
		})?;

		let members = Self::votable_member_sorted();
		T::MembershipChanged::change_members_sorted(&[who.clone()], &[], &members[..]);
		Ok(())
	}

	/// Remove a user from the alliance member set.
	fn remove_member(who: &T::AccountId, role: MemberRole) -> DispatchResult {
		<Members<T, I>>::mutate(role, |members| -> DispatchResult {
			let pos = members
				.binary_search(who)
				.ok()
				.ok_or(Error::<T, I>::NotMember)?;
			members.remove(pos);
			Ok(())
		})?;

		let members = Self::votable_member_sorted();
		T::MembershipChanged::change_members_sorted(&[], &[who.clone()], &members[..]);
		Ok(())
	}

	/// Check if a user is in blacklist.
	fn is_blacklist(info: &UserIdentity<T::AccountId>) -> bool {
		match info {
			UserIdentity::Website(url) => <WebsiteBlacklist<T, I>>::get().contains(url),
			UserIdentity::AccountId(who) => <AccountBlacklist<T, I>>::get().contains(who),
		}
	}

	/// Check if a user is in account blacklist.
	fn is_account_blacklist(who: &T::AccountId) -> bool {
		<AccountBlacklist<T, I>>::get().contains(who)
	}

	/// Add a identity info to the blacklist set.
	fn do_add_blacklist(
		new_accounts: &mut Vec<T::AccountId>,
		new_webs: &mut Vec<Url>,
	) -> DispatchResult {
		if !new_accounts.is_empty() {
			let mut users = <AccountBlacklist<T, I>>::get();
			users.append(new_accounts);
			users.sort();
			AccountBlacklist::<T, I>::put(users);
		}
		if !new_webs.is_empty() {
			let mut webs = <WebsiteBlacklist<T, I>>::get();
			webs.append(new_webs);
			webs.sort();
			WebsiteBlacklist::<T, I>::put(webs);
		}
		Ok(())
	}

	/// Remove a identity info from the blacklist.
	fn do_remove_blacklist(
		out_accounts: &mut Vec<T::AccountId>,
		out_webs: &mut Vec<Url>,
	) -> DispatchResult {
		if !out_accounts.is_empty() {
			let mut accounts = <AccountBlacklist<T, I>>::get();
			for who in out_accounts.iter() {
				let pos = accounts
					.binary_search(who)
					.ok()
					.ok_or(Error::<T, I>::NotInBlacklist)?;
				accounts.remove(pos);
			}
			AccountBlacklist::<T, I>::put(accounts);
		}
		if !out_webs.is_empty() {
			let mut webs = <WebsiteBlacklist<T, I>>::get();
			for web in out_webs.iter() {
				let pos = webs
					.binary_search(web)
					.ok()
					.ok_or(Error::<T, I>::NotInBlacklist)?;
				webs.remove(pos);
			}
			WebsiteBlacklist::<T, I>::put(webs);
		}
		Ok(())
	}

	fn has_identity(who: &T::AccountId) -> bool {
		T::IdentityVerifier::verify_identity(who, IDENTITY_FIELD_DISPLAY)
			&& T::IdentityVerifier::verify_identity(who, IDENTITY_FIELD_WEB)
	}
}

pub trait IdentityVerifier<AccountId: Clone + Ord> {
	fn verify_identity(who: &AccountId, fields: u64) -> bool;
}

pub trait ProposalProvider<AccountId, Hash, Proposal> {
	fn propose_proposal(
		who: AccountId,
		threshold: u32,
		proposal: Proposal,
		proposal_hash: Hash,
	) -> Result<u32, DispatchError>;

	fn veto_proposal(proposal_hash: Hash) -> u32;

	fn close_proposal(
		proposal_hash: Hash,
		index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> Result<(Weight, Pays), DispatchError>;

	fn proposal_of(proposal_hash: Hash) -> Option<Proposal>;
}
