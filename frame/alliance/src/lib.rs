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

use sp_runtime::{
    traits::{Bounded, StaticLookup, Zero, Hash},
    RuntimeDebug,
};
use sp_std::prelude::*;

use frame_support::{
    codec::{Decode, Encode},
    dispatch::{
        DispatchError, DispatchResult, Dispatchable, Parameter,
        PostDispatchInfo, GetDispatchInfo,
    },
    ensure,
    weights::{Weight, Pays},
    traits::{ChangeMembers, OnUnbalanced, Currency, Get, LockableCurrency, ReservableCurrency, InitializeMembers, IsType, IsSubType},
};
use frame_system::ensure_signed;
pub use pallet::*;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

type BalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

const IDENTITY_FIELD_DISPLAY: u64 = 0b0000000000000000000000000000000000000000000000000000000000000001;
const IDENTITY_FIELD_WEB: u64 = 0b0000000000000000000000000000000000000000000000000000000000000100;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MemberRole {
    Founder,
    Fellow,
    /// The current set of allies; candidates who have been approved but have not been elevated to fellow.
    /// Allies can’t propose proposal or vote on motions, only for show.
    Ally,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MemberIdentity<AccountId> {
    /// The member has been chosen to be skeptic and has not yet taken any action.
    Website(Vec<u8>),
    /// The member has rejected the candidate's application.
    Address(AccountId),
}

/// Application form to become a candidate to entry into alliance.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct CandidacyForm<AccountId, Balance> {
    /// The outsider trying to enter alliance
    who: AccountId,
    /// The kind of candidacy placed for this outsider.
    kind: CandidacyKind<AccountId, Balance>,
}

/// A vote by a member on a candidate application.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum CandidacyKind<AccountId, Balance> {
    /// The CandidateDeposit was paid for this submit candidacy.
    Submit(Balance),
    /// A fellow/founder nominate candidacy for a outsider with zero deposit.
    Nominate(AccountId),
}

/// Details surrounding a specific instance of an announcement to make a call.
#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct Announcement<AccountId, BlockNumber> {
    /// The account which made the announcement.
    publisher: AccountId,
    /// The hash of the call to be made.
    content: cid::Cid,
    /// The height at which the announcement was made.
    height: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    /// The module configuration trait.
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// The outer call dispatch type.
        type Proposal: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo>
        + GetDispatchInfo + From<frame_system::Call<Self>> + IsSubType<Call<Self, I>>
        + IsType<<Self as frame_system::Config>::Call>;

        /// The origin that is allowed to call `init_founder`.
        type FounderInitOrigin: EnsureOrigin<Self::Origin>;

        /// Origin from which the next tabled referendum may be forced; this allows for the tabling of
        /// a majority-carries referendum.
        type MajorityOrigin: EnsureOrigin<Self::Origin>;

        /// The currency used for deposits.
        type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>
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
        type CandidateDeposit: Get<BalanceOf<Self, I>>;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        AlreadyCandidate,
        AlreadyMember,
        AlreadyElevated,
        AlreadyInBlacklist,
        NotCandidate,
        NotAlly,
        NotFounder,
        NotMember,
        NotElevatedMember,
        SuspendedMember,
        InsufficientCandidateFunds,
        NotInBlacklist,
        NoIdentity,
        ProposalMissing,
        ProposalNotVetoable,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        AllianceRuleUpdate(cid::Cid),
        CandidateAdded(T::AccountId, CandidacyKind<T::AccountId, BalanceOf<T, I>>),
        FoundersInit(Vec<T::AccountId>),
        MemberAdded(T::AccountId, MemberRole),
        MemberRetire(T::AccountId),
        MemberKicked(T::AccountId),
        BlacklistAdded(MemberIdentity<T::AccountId>),
        BlacklistRemoved(MemberIdentity<T::AccountId>),
        AnnouncementPublish(T::AccountId, cid::Cid),
        PrimeSet(Option<T::AccountId>),
    }

    /// A ipfs cid of the rules of this alliance concerning membership.
    /// Any member can propose rules, other members make a traditional majority-wins
    /// vote to determine if the rules take effect.
    /// The founder has a special one-vote veto right to the rules setting.
    #[pallet::storage]
    #[pallet::getter(fn rules)]
    pub type Rules<T: Config<I>, I: 'static = ()> = StorageValue<_, cid::Cid, OptionQuery>;

    /// Maps proposal hash and identity info.
    #[pallet::storage]
    #[pallet::getter(fn announcements)]
    pub type Announcements<T: Config<I>, I: 'static = ()> =
    StorageValue<_, Vec<Announcement<T::AccountId, T::BlockNumber>>, ValueQuery>;

    /// The member who have locked a candidate deposit.
    #[pallet::storage]
    #[pallet::getter(fn deposit_of)]
    pub type DepositOf<T: Config<I>, I: 'static = ()> =
    StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T, I>, OptionQuery>;

    /// The current set of candidates; outsiders who are attempting to become members.
    #[pallet::storage]
    #[pallet::getter(fn candidates)]
    pub type Candidates<T: Config<I>, I: 'static = ()> =
    StorageValue<_, Vec<CandidacyForm<T::AccountId, BalanceOf<T, I>>>, ValueQuery>;

    /// The current set of alliance members(founder/fellow/ally).
    ///
    /// Note: ally can’t proposal or vote on motions, only for show.
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config<I>, I: 'static = ()> =
    StorageMap<_, Twox64Concat, MemberRole, Vec<T::AccountId>, ValueQuery>;

    /// The set of suspended members.
    #[pallet::storage]
    #[pallet::getter(fn suspended_member)]
    pub type SuspendedMembers<T: Config<I>, I: 'static = ()> =
    StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

    /// Maps proposal hash and identity info.
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config<I>, I: 'static = ()> =
    StorageValue<_, Vec<(T::BlockNumber, MemberIdentity<T::AccountId>)>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub founders: Vec<T::AccountId>,
        pub fellows: Vec<T::AccountId>,
        pub phantom: PhantomData<(T, I)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                founders: Vec::new(),
                fellows: Vec::new(),
                phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            if !self.founders.is_empty() {
                assert!(<Members<T, I>>::get(MemberRole::Founder).is_empty(), "Founders are already initialized!");
                Members::<T, I>::insert(MemberRole::Founder, self.founders.clone());
            }
            if !self.fellows.is_empty() {
                assert!(<Members<T, I>>::get(MemberRole::Fellow).is_empty(), "Fellows are already initialized!");
                Members::<T, I>::insert(MemberRole::Fellow, self.fellows.clone());
            }
        }
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Add a new proposal to be voted on.
        ///
        /// Requires the sender to be elevated member(founders/fellows).
        #[pallet::weight(0)]
        pub(super) fn propose(origin: OriginFor<T>, proposal: Box<<T as Config<I>>::Proposal>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_elevated_member(&who), Error::<T, I>::NotElevatedMember);
            ensure!(!<SuspendedMembers<T, I>>::contains_key(&who), Error::<T, I>::SuspendedMember);

            let proposal_hash = T::Hashing::hash_of(&proposal);
            match proposal.is_sub_type() {
                Some(Call::kick_member(ref strike)) => {
                    <SuspendedMembers<T, I>>::insert(strike, true);
                }
                Some(Call::add_blacklist(ref info)) => {
                    if let MemberIdentity::Address(strike) = info {
                        <SuspendedMembers<T, I>>::insert(strike, true);
                    }
                }
                _ => ()
            }

            // The motion with high bound of (2/3f+1) needed.
            let threshold = 2 * Self::elevated_member_count() / 3 + 1;
            T::ProposalProvider::propose_proposal(who, threshold, *proposal, proposal_hash)?;
            Ok(())
        }

        /// Disapprove a proposal, close, and remove it from the system, regardless of its current state.
        ///
        /// Must be called by the founders.
        #[pallet::weight(0)]
        pub(super) fn veto(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_member(&who, Some(MemberRole::Founder)).0, Error::<T, I>::NotFounder);
            ensure!(!<SuspendedMembers<T, I>>::contains_key(&who), Error::<T, I>::SuspendedMember);

            let proposal = T::ProposalProvider::proposal_of(proposal_hash);
            ensure!(proposal.is_some(), Error::<T, I>::ProposalMissing);
            let veto_rights = match proposal.unwrap().is_sub_type() {
                Some(Call::set_rule(..)) | Some(Call::elevate_ally(..)) => true,
                _ => false
            };
            ensure!(veto_rights, Error::<T, I>::ProposalNotVetoable);

            T::ProposalProvider::veto_proposal(proposal_hash);
            Ok(())
        }

        #[pallet::weight(0)]
        pub(super) fn close(origin: OriginFor<T>,
                            proposal_hash: T::Hash,
                            index: ProposalIndex,
                            proposal_weight_bound: Weight,
                            length_bound: u32,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            let proposal = T::ProposalProvider::proposal_of(proposal_hash);
            ensure!(proposal.is_some(),  Error::<T, I>::ProposalMissing);

            let (_, pays) = T::ProposalProvider::close_proposal(proposal_hash, index, proposal_weight_bound, length_bound)?;
            if Pays::No == pays {
                match proposal.unwrap().is_sub_type() {
                    Some(Call::kick_member(ref strike)) => {
                        <SuspendedMembers<T, I>>::remove(strike);
                    }
                    Some(Call::add_blacklist(ref info)) => {
                        if let MemberIdentity::Address(strike) = info {
                            <SuspendedMembers<T, I>>::remove(strike);
                        }
                    }
                    _ => ()
                }
            }

            Ok(())
        }

        /// Initialize the founders to the given members.
        #[pallet::weight(0)]
        pub(super) fn init_founders(origin: OriginFor<T>, founders: Vec<T::AccountId>, prime: Option<T::AccountId>) -> DispatchResult {
            T::FounderInitOrigin::ensure_origin(origin)?;
            let mut founders = founders.clone();
            founders.sort();
            T::InitializeMembers::initialize_members(&founders);
            T::MembershipChanged::set_prime(prime);
            Members::<T, I>::insert(&MemberRole::Founder, founders.clone());
            Self::deposit_event(Event::FoundersInit(founders));
            Ok(())
        }

        /// Set the prime member.
        #[pallet::weight(0)]
        pub(super) fn set_prime(origin: OriginFor<T>, prime: Option<T::AccountId>) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;

            T::MembershipChanged::set_prime(prime.clone());
            Self::deposit_event(Event::PrimeSet(prime));
            Ok(())
        }

        /// A IPFS cid of the rules of this alliance concerning membership.
        #[pallet::weight(0)]
        pub(super) fn set_rule(origin: OriginFor<T>, rule: cid::Cid) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;

            Rules::<T, I>::put(&rule);
            Self::deposit_event(Event::AllianceRuleUpdate(rule));
            Ok(())
        }

        /// Announcement IPFS Hash about dispute between two allies and other issues.
        /// Proposer should publish in polkassembly.io first and talked with others,
        /// then publish the post into IPFS. Create a ID.
        #[pallet::weight(0)]
        pub(super) fn announce(origin: OriginFor<T>, publisher: T::AccountId, announcement: cid::Cid) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;

            let mut announcements = <Announcements<T, I>>::get();
            announcements.push(Announcement {
                publisher: publisher.clone(),
                content: announcement.clone(),
                height: T::BlockNumber::max_value(),
            });
            <Announcements<T, I>>::put(&announcements);
            Self::deposit_event(Event::AnnouncementPublish(publisher, announcement));
            Ok(())
        }

        /// Submit oneself for candidacy.
        ///
        /// Account must have enough transferable funds in it to pay the candidate deposit.
        #[pallet::weight(0)]
        pub(super) fn submit_candidacy(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let candidates = <Candidates<T, I>>::get();
            ensure!(!Self::is_candidate(&candidates, &who),Error::<T, I>::AlreadyCandidate);
            ensure!(!Self::is_member(&who, None).0, Error::<T, I>::AlreadyMember);
            ensure!(!Self::is_blacklist(&MemberIdentity::Address(who.clone())), Error::<T, I>::AlreadyInBlacklist);

            // check user self or parent should has verified identity to reuse display name and website.
            ensure!(T::IdentityVerifier::verify_identity(who.clone(), IDENTITY_FIELD_DISPLAY) &&
                T::IdentityVerifier::verify_identity(who.clone(), IDENTITY_FIELD_WEB), Error::<T, I>::NoIdentity);

            let deposit = T::CandidateDeposit::get();
            T::Currency::reserve(&who, deposit).map_err(|_| Error::<T, I>::InsufficientCandidateFunds)?;

            Self::add_candidate(CandidacyForm { who: who.clone(), kind: CandidacyKind::Submit(deposit) })?;

            Self::deposit_event(Event::CandidateAdded(who, CandidacyKind::Submit(deposit)));
            Ok(())
        }

        /// As a elevated member, nominate for someone to join alliance.
        ///
        /// There is no deposit required to the nominees.
        ///
        /// The dispatch origin for this call must be _Signed_ and a elevated member.
        #[pallet::weight(0)]
        pub(super) fn nominate_candidacy(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
            let nominator = ensure_signed(origin)?;
            ensure!(Self::is_elevated_member(&nominator), Error::<T, I>::NotElevatedMember);
            ensure!(!<SuspendedMembers<T, I>>::contains_key(&nominator), Error::<T, I>::SuspendedMember);
            let candidates = <Candidates<T, I>>::get();
            ensure!(!Self::is_candidate(&candidates, &who),Error::<T, I>::AlreadyCandidate);
            ensure!(!Self::is_member(&who, None).0, Error::<T, I>::AlreadyMember);
            ensure!(!Self::is_blacklist(&MemberIdentity::Address(who.clone())), Error::<T, I>::AlreadyInBlacklist);

            // check user self or parent should has verified identity to reuse display name and website.
            ensure!(T::IdentityVerifier::verify_identity(who.clone(), IDENTITY_FIELD_DISPLAY) &&
                T::IdentityVerifier::verify_identity(who.clone(), IDENTITY_FIELD_WEB), Error::<T, I>::NoIdentity);

            Self::add_candidate(CandidacyForm { who: who.clone(), kind: CandidacyKind::Nominate(nominator.clone()) })?;

            Self::deposit_event(Event::CandidateAdded(who, CandidacyKind::Nominate(nominator)));
            Ok(())
        }

        /// vote a candidate to ally.
        #[pallet::weight(0)]
        pub(super) fn vote_candidate(origin: OriginFor<T>, candidate: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;
            let candidate = T::Lookup::lookup(candidate)?;
            let candidates = <Candidates<T, I>>::get();
            ensure!(Self::is_candidate(&candidates, &candidate), Error::<T, I>::NotCandidate);
            ensure!(!Self::is_member(&candidate, None).0, Error::<T, I>::AlreadyMember);
            ensure!(!Self::is_blacklist(&MemberIdentity::Address(candidate.clone())), Error::<T, I>::AlreadyInBlacklist);

            match Self::remove_candidate(&candidate)? {
                CandidacyKind::Submit(deposit) => {
                    <DepositOf<T, I>>::insert(&candidate, deposit);
                }
                _ => {}
            }

            let role = MemberRole::Ally;
            Self::add_member(&candidate, role)?;
            Self::deposit_event(Event::MemberAdded(candidate, role));
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
            T::MajorityOrigin::ensure_origin(origin)?;
            let ally = T::Lookup::lookup(ally)?;
            ensure!(Self::is_member(&ally, Some(MemberRole::Ally)).0,Error::<T, I>::NotAlly);
            ensure!(!Self::is_elevated_member(&ally), Error::<T, I>::AlreadyElevated);
            ensure!(!<SuspendedMembers<T, I>>::contains_key(&ally), Error::<T, I>::SuspendedMember);

            let role = MemberRole::Fellow;
            Self::remove_member(ally.clone(), MemberRole::Ally)?;
            Self::add_member(&ally, role)?;
            Self::deposit_event(Event::MemberAdded(ally, role));
            Ok(())
        }

        /// As a member, back to outsider and unlock deposit.
        #[pallet::weight(0)]
        pub(super) fn retire(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let (is_member, role) = Self::is_member(&who, None);
            ensure!(is_member, Error::<T, I>::NotMember);
            ensure!(!<SuspendedMembers<T, I>>::contains_key(&who), Error::<T, I>::SuspendedMember);

            Self::remove_member(who.clone(), role.unwrap())?;
            if let Some(deposit) = DepositOf::<T, I>::take(who.clone()) {
                let err_amount = T::Currency::unreserve(&who, deposit);
                debug_assert!(err_amount.is_zero());
            }

            Self::deposit_event(Event::MemberRetire(who));
            Ok(())
        }

        /// Kick a member to outsider with its deposit slashed.
        #[pallet::weight(0)]
        pub(super) fn kick_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;

            let (is_member, role) = Self::is_member(&who, None);
            ensure!(is_member, Error::<T, I>::NotMember);

            Self::remove_member(who.clone(), role.unwrap())?;

            if let Some(deposit) = DepositOf::<T, I>::take(who.clone()) {
                T::Slashed::on_unbalanced(T::Currency::slash_reserved(&who, deposit).0);
            }

            Self::deposit_event(Event::MemberKicked(who));
            Ok(())
        }

        /// Add websites or addresses into blacklist.
        #[pallet::weight(0)]
        pub(super) fn add_blacklist(origin: OriginFor<T>, info: MemberIdentity<T::AccountId>) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_blacklist(&info), Error::<T, I>::AlreadyInBlacklist);

            Self::blacklist_add(T::BlockNumber::max_value(), &info)?;
            Self::deposit_event(Event::BlacklistAdded(info));
            Ok(())
        }

        /// Remove websites or addresses form blacklist.
        #[pallet::weight(0)]
        pub(super) fn remove_blacklist(origin: OriginFor<T>, info: MemberIdentity<T::AccountId>) -> DispatchResult {
            T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(Self::is_blacklist(&info), Error::<T, I>::NotInBlacklist);

            Self::blacklist_remove(&info)?;
            Self::deposit_event(Event::BlacklistRemoved(info));
            Ok(())
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Check if a user is a candidate.
    fn is_candidate(
        candidates: &Vec<CandidacyForm<T::AccountId, BalanceOf<T, I>>>,
        who: &T::AccountId,
    ) -> bool {
        candidates
            .binary_search_by_key(who, |a| a.who.clone())
            .is_ok()
    }

    /// Add a candidate to the sorted candidate list.
    fn add_candidate(form: CandidacyForm<T::AccountId, BalanceOf<T, I>>) -> DispatchResult {
        let mut candidates = <Candidates<T, I>>::get();
        let insert_position = candidates
            .binary_search_by(|a| a.who.cmp(&form.who))
            .err()
            .ok_or(Error::<T, I>::AlreadyCandidate)?;
        candidates.insert(insert_position, form);
        Candidates::<T, I>::put(candidates);
        Ok(())
    }

    /// Remove a candidate from the candidates list.
    fn remove_candidate(
        who: &T::AccountId,
    ) -> sp_std::result::Result<CandidacyKind<T::AccountId, BalanceOf<T, I>>, DispatchError> {
        let mut candidates = <Candidates<T, I>>::get();
        let position = candidates
            .binary_search_by_key(who, |a| a.who.clone())
            .ok()
            .ok_or(Error::<T, I>::NotCandidate)?;
        let candidate = candidates.remove(position);
        Candidates::<T, I>::put(candidates);
        Ok(candidate.kind)
    }

    fn elevated_member_count() -> u32 {
        let founders = Members::<T, I>::get(MemberRole::Founder);
        let fellows = Members::<T, I>::get(MemberRole::Fellow);
        (founders.len() + fellows.len()) as u32
    }

    fn is_elevated_member(who: &T::AccountId) -> bool {
        Self::is_member(who, Some(MemberRole::Founder)).0
            || Self::is_member(who, Some(MemberRole::Fellow)).0
    }

    /// Check if a user is a alliance member.
    fn is_member(who: &T::AccountId, role: Option<MemberRole>) -> (bool, Option<MemberRole>) {
        if let Some(role) = role {
            let members = Members::<T, I>::get(&role);
            (members.binary_search(&who).is_ok(), Some(role))
        } else {
            let roles: Vec<_> = Members::<T, I>::iter()
                .filter(|(_, v)| v.binary_search(who).is_ok())
                .map(|(k, _)| k)
                .collect();
            if roles.len() >= 1 {
                (true, Some(roles[0]))
            } else {
                (false, None)
            }
        }
    }

    /// Add a user to the sorted alliance member set.
    fn add_member(who: &T::AccountId, role: MemberRole) -> DispatchResult {
        <Members<T, I>>::mutate(role, |members| -> DispatchResult {
            let insert_position = members
                .binary_search(&who)
                .err()
                .ok_or(Error::<T, I>::AlreadyMember)?;
            members.insert(insert_position, who.clone());
            if role == MemberRole::Founder || role == MemberRole::Fellow {
                T::MembershipChanged::change_members_sorted(&[who.clone()], &[], members);
            }
            Ok(())
        })
    }

    /// Remove a user from the alliance member set.
    fn remove_member(who: T::AccountId, role: MemberRole) -> DispatchResult {
        <Members<T, I>>::mutate(role, |members| -> DispatchResult {
            let position = members
                .binary_search(&who)
                .ok()
                .ok_or(Error::<T, I>::NotMember)?;
            members.remove(position);
            if role == MemberRole::Founder || role == MemberRole::Fellow {
                T::MembershipChanged::change_members_sorted(&[], &[who.clone()], members);
            }
            Ok(())
        })
    }

    /// Check if a identity info is in blacklist.
    fn is_blacklist(info: &MemberIdentity<T::AccountId>) -> bool {
        let blacklist = <Blacklist<T, I>>::get();
        blacklist.iter().find(|i| i.1 == *info).is_some()
    }

    /// Add a identity info to the blacklist set.
    fn blacklist_add(number: T::BlockNumber, info: &MemberIdentity<T::AccountId>) -> DispatchResult {
        let mut blacklist = <Blacklist<T, I>>::get();
        let insert_position = blacklist
            .binary_search_by(|&(a, _)| a.cmp(&number))
            .err()
            .ok_or(Error::<T, I>::AlreadyInBlacklist)?;
        blacklist.insert(insert_position, (number, info.clone()));
        <Blacklist<T, I>>::put(&blacklist);
        Ok(())
    }

    /// Remove a identity `info` from the blacklist.
    fn blacklist_remove(info: &MemberIdentity<T::AccountId>) -> DispatchResult {
        let mut blacklist = <Blacklist<T, I>>::get();
        let position = blacklist
            .iter()
            .position(|(_, a)| a == info)
            .ok_or(Error::<T, I>::NotInBlacklist)?;
        blacklist.remove(position);
        <Blacklist<T, I>>::put(&blacklist);
        Ok(())
    }
}

pub trait IdentityVerifier<AccountId: Clone + Ord> {
    fn verify_identity(who: AccountId, fields: u64) -> bool;
}

pub trait ProposalProvider<AccountId, Hash, Proposal> {
    fn propose_proposal(who: AccountId, threshold: u32, proposal: Proposal,
                        proposal_hash: Hash) -> Result<u32, DispatchError>;

    fn veto_proposal(proposal_hash: Hash) -> u32;

    fn close_proposal(proposal_hash: Hash,
                      index: ProposalIndex,
                      proposal_weight_bound: Weight,
                      length_bound: u32,
    ) -> Result<(Weight, Pays), DispatchError>;

    fn proposal_of(proposal_hash: Hash) -> Option<Proposal>;
}
