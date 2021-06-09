//! # Identity Pallet
//!
//! TODO: description
//!
//! - [`Pallet`]
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//!
//! [`Pallet`]: ./struct.Pallet.html
//! [`Config`]: ./trait.Config.html
//! [`Call`]: ./enum.Call.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod mock;
mod tests;
mod weights;

pub use self::pallet::*;
pub use self::weights::*;

use cid::Cid;

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{
        DispatchError, DispatchResult, DispatchResultWithPostInfo, Dispatchable, GetDispatchInfo,
        PostDispatchInfo,
    },
    traits::{
        ChangeMembers, Currency, EnsureOrigin, Get, InitializeMembers, IsSubType, OnUnbalanced,
        ReservableCurrency,
    },
    weights::Weight,
};
// use sp_core::u32_trait::Value as U32;
use sp_runtime::{
    traits::{Bounded, StaticLookup},
    RuntimeDebug,
};
use sp_std::marker::PhantomData;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

///
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode)]
pub enum MemberRole {
    Ally,
    Fellow,
    Founder,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode)]
pub struct MemberInfo<T: Config> {
    ///
    pub role: MemberRole,
    ///
    pub candidacy: Option<CandidacyKind<T::AccountId, BalanceOf<T>>>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode)]
pub struct CandidateInfo<T: Config> {
    ///
    pub candidacy: CandidacyKind<T::AccountId, BalanceOf<T>>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode)]
pub enum CandidacyKind<AccountId, Balance> {
    Submit(Balance),
    Nominate(AccountId),
}

pub type MemberCount = u32;
pub type ProposalIndex = u32;

pub trait MotionOperationProvider<AccountId, Hash, Proposal> {
    fn propose(
        who: AccountId,
        threshold: MemberCount,
        proposal: Proposal,
    ) -> DispatchResultWithPostInfo;

    fn vote(
        who: AccountId,
        proposal: Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> DispatchResultWithPostInfo;

    fn close(
        proposal_hash: Hash,
        index: ProposalIndex,
        proposal_weight_bound: Weight,
        length_bound: u32,
    ) -> DispatchResultWithPostInfo;

    fn veto(proposal_hash: Hash) -> DispatchResultWithPostInfo;

    fn proposal_of(proposal_hash: Hash) -> Option<Proposal>;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// `frame_system::Config` should always be included.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + From<frame_system::Call<Self>>
            + GetDispatchInfo
            + IsSubType<Call<Self>>;

        ///
        type MotionOperation: MotionOperationProvider<Self::AccountId, Self::Hash, Self::Proposal>;

        /// The receiver of the signal for when the membership has been initialized.
        type MembershipInitialized: InitializeMembers<Self::AccountId>;

        /// The receiver of the signal for when the membership has changed.
        type MembershipChanged: ChangeMembers<Self::AccountId>;

        ///
        type MajorityOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

        /// The currency type used for this pallet.
        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type CandidateDeposit: Get<BalanceOf<Self>>;

        /// Handler for the unbalanced decrease when slashing.
        type OnSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        founders: Vec<T::AccountId>,
        _marker: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                founders: Vec::new(),
                _marker: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Members::<T>::insert(MemberRole::Founder, self.founders.clone());
        }
    }

    /// The current set of candidates; outsiders that are attempting to become members.
    #[pallet::storage]
    #[pallet::getter(fn candidate_info_of)]
    pub type CandidateInfoOf<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CandidateInfo<T>, OptionQuery>;

    ///
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> =
        StorageMap<_, Twox64Concat, MemberRole, Vec<T::AccountId>, ValueQuery>;

    ///
    #[pallet::storage]
    #[pallet::getter(fn member_info_of)]
    pub type MemberInfoOf<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, MemberInfo<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn suspended)]
    pub type Suspended<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    ///
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// A IPFS hash of the rules of this alliance concerning membership.
    #[pallet::storage]
    #[pallet::getter(fn rules)]
    pub type Rules<T: Config> = StorageValue<_, Cid, OptionQuery>;

    /// A IPFS hash of the post about dispute between two allies and other issues.
    #[pallet::storage]
    #[pallet::getter(fn announcement)]
    pub type Announcements<T: Config> =
        StorageMap<_, Blake2_128Concat, Cid, T::AccountId, ValueQuery>;

    /// Alliance pallet event;
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FounderInitialized(Vec<T::AccountId>),
        CandidateSubmitted(T::AccountId, BalanceOf<T>),
        CandidateNominated(T::AccountId, T::AccountId),
        RulesSet(T::AccountId, Cid),
        Announced(T::AccountId, Cid),
        CandidateApproved(T::AccountId, T::AccountId),
        CandidateRejected(T::AccountId, T::AccountId),
        AllyElevated(T::AccountId, T::AccountId),
        MemberRetired(
            T::AccountId,
            Option<CandidacyKind<T::AccountId, BalanceOf<T>>>,
        ),
        MemberKicked(
            T::AccountId,
            T::AccountId,
            Option<CandidacyKind<T::AccountId, BalanceOf<T>>>,
        ),
        BlacklistAdded(Vec<T::AccountId>),
        BlacklistRemoved(Vec<T::AccountId>),
    }

    /// Alliance pallet error
    #[pallet::error]
    pub enum Error<T> {
        InsufficientCandidateDeposit,
        AlreadyInitializedFounder,

        AlreadySuspended,

        MissingProposalHash,
        NotVetoableProposal,

        AlreadyCandidate,
        NotCandidate,

        AlreadyMember,
        NotMember,
        AlreadyAllyMember,
        NotAllyMember,
        AlreadyElevatedMember,
        NotElevatedMember,
        AlreadyFounderMember,
        NotFounderMember,

        AlreadyInBlacklist,
        NotInBlacklist,
    }

    /// Alliance pallet declaration
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        ///
        #[pallet::weight(0)]
        pub(crate) fn init_founders(
            origin: OriginFor<T>,
            founders: Vec<T::AccountId>,
        ) -> DispatchResult {
            let _ = ensure_root(origin)?;
            ensure!(
                !Self::has_member(MemberRole::Founder),
                Error::<T>::AlreadyInitializedFounder
            );

            let mut founders = founders;
            founders.sort();
            Members::<T>::insert(MemberRole::Founder, founders.clone());
            let info = MemberInfo {
                role: MemberRole::Founder,
                candidacy: None,
            };
            founders
                .iter()
                .for_each(|founder| MemberInfoOf::<T>::insert(founder, info.clone()));
            T::MembershipInitialized::initialize_members(&founders);

            Self::deposit_event(Event::FounderInitialized(founders));
            Ok(())
        }

        ///
        #[pallet::weight(0)]
        pub(crate) fn propose(
            origin: OriginFor<T>,
            proposal: Box<<T as Config>::Proposal>,
        ) -> DispatchResultWithPostInfo {
            let proposor = ensure_signed(origin)?;
            ensure!(
                Self::is_votale_member(&proposor),
                Error::<T>::NotElevatedMember
            );
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            // let proposal_hash = T::Hashing::hash_of(&proposal);
            match proposal.is_sub_type() {
                Some(Call::kick_member(who))
                | Some(Call::approve_candidate(who))
                | Some(Call::reject_candidate(who)) => {
                    let who = T::Lookup::lookup(who.clone())?;
                    <Suspended<T>>::insert(who, true);
                }
                Some(Call::add_blacklist(_)) | Some(Call::remove_blacklist(_)) => {}
                _ => {}
            }

            // maybe use config to set the threshold
            // let threshold = (2/3f+1);
            T::MotionOperation::propose(proposor, /*threshold*/ 3, *proposal)
        }

        #[pallet::weight(0)]
        fn vote(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            index: ProposalIndex,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let proposor = ensure_signed(origin)?;
            ensure!(
                Self::is_votale_member(&proposor),
                Error::<T>::NotElevatedMember
            );
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let proposal = T::MotionOperation::proposal_of(proposal_hash);
            ensure!(proposal.is_some(), Error::<T>::MissingProposalHash);

            T::MotionOperation::vote(proposor, proposal_hash, index, approve)
        }

        #[pallet::weight(0)]
        fn close(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            index: ProposalIndex,
            proposal_weight_bound: Weight,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let proposor = ensure_signed(origin)?;
            ensure!(
                Self::is_votale_member(&proposor),
                Error::<T>::NotElevatedMember
            );
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let proposal = T::MotionOperation::proposal_of(proposal_hash);
            ensure!(proposal.is_some(), Error::<T>::MissingProposalHash);

            let dispatch_info = T::MotionOperation::close(
                proposal_hash,
                index,
                proposal_weight_bound,
                length_bound,
            )?;
            if dispatch_info.pays_fee == Pays::No {
                match proposal.expect("proposal must be exist; qed").is_sub_type() {
                    Some(Call::kick_member(who))
                    | Some(Call::approve_candidate(who))
                    | Some(Call::reject_candidate(who)) => {
                        let who = T::Lookup::lookup(who.clone())?;
                        <Suspended<T>>::remove(who);
                    }
                    Some(Call::add_blacklist(_)) | Some(Call::remove_blacklist(_)) => {}
                    _ => {}
                }
            }

            Ok(dispatch_info)
        }

        #[pallet::weight(0)]
        pub(crate) fn veto(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_founder(&who), Error::<T>::NotFounderMember);
            ensure!(!Self::is_suspended(&who), Error::<T>::AlreadySuspended);

            let proposal = T::MotionOperation::proposal_of(proposal_hash);
            ensure!(proposal.is_some(), Error::<T>::MissingProposalHash);

            match proposal.expect("proposal must be exist; qed").is_sub_type() {
                Some(Call::elevate_ally(_)) | Some(Call::set_rules(_)) => {
                    T::MotionOperation::veto(proposal_hash)
                }
                _ => Err(Error::<T>::NotVetoableProposal.into()),
            }
        }

        /// Become a `Candidate`.
        /// It is permissionless but itself or parent should has verified identity to
        /// reuse display name and website.
        #[pallet::weight(0)]
        pub(crate) fn submit_candidate(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Self::is_candidate(&who), Error::<T>::AlreadyCandidate);
            ensure!(!Self::is_member(&who), Error::<T>::AlreadyMember);
            ensure!(!Self::is_blacklist(&who), Error::<T>::AlreadyInBlacklist);

            let deposit = T::CandidateDeposit::get();
            ensure!(
                T::Currency::can_reserve(&who, deposit),
                Error::<T>::InsufficientCandidateDeposit
            );
            T::Currency::reserve(&who, deposit)?;
            CandidateInfoOf::<T>::insert(
                &who,
                CandidateInfo {
                    candidacy: CandidacyKind::Submit(deposit),
                },
            );

            Self::deposit_event(Event::CandidateSubmitted(who, deposit));
            Ok(())
        }

        // Nominate `Outsider` to `Candidate` with 0 deposit.
        #[pallet::weight(0)]
        pub(crate) fn nominate_candidate(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let nominator = ensure_signed(origin)?;
            ensure!(
                Self::is_votale_member(&nominator),
                Error::<T>::NotElevatedMember
            );
            ensure!(
                !Self::is_suspended(&nominator),
                Error::<T>::AlreadySuspended
            );

            let who = T::Lookup::lookup(who)?;
            ensure!(!Self::is_candidate(&who), Error::<T>::AlreadyCandidate);
            ensure!(!Self::is_member(&who), Error::<T>::AlreadyMember);
            ensure!(!Self::is_blacklist(&who), Error::<T>::AlreadyInBlacklist);

            CandidateInfoOf::<T>::insert(
                &who,
                CandidateInfo {
                    candidacy: CandidacyKind::Nominate(nominator.clone()),
                },
            );

            Self::deposit_event(Event::CandidateNominated(nominator, who));
            Ok(())
        }

        ///
        #[pallet::weight(0)]
        pub(crate) fn set_rules(origin: OriginFor<T>, rules: Cid) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            Rules::<T>::mutate(|r| {
                *r = Some(rules.clone());
            });

            Self::deposit_event(Event::RulesSet(proposor, rules));
            Ok(())
        }

        ///
        #[pallet::weight(0)]
        pub(crate) fn announce(origin: OriginFor<T>, announcement: Cid) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            Announcements::<T>::insert(announcement.clone(), proposor.clone());

            Self::deposit_event(Event::Announced(proposor, announcement));
            Ok(())
        }

        /// Approve a `Candidate` to be a `Ally`.
        /// Only the members (`Fellows` and `Founders`) can vote to approve/reject the `Candidate`.
        #[pallet::weight(0)]
        pub(crate) fn approve_candidate(
            origin: OriginFor<T>,
            candidate: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let candidate = T::Lookup::lookup(candidate)?;
            ensure!(Self::is_candidate(&candidate), Error::<T>::NotCandidate);
            ensure!(
                !Self::is_suspended(&candidate),
                Error::<T>::AlreadySuspended
            );
            ensure!(!Self::is_member(&candidate), Error::<T>::AlreadyMember);
            ensure!(
                !Self::is_blacklist(&candidate),
                Error::<T>::AlreadyInBlacklist
            );

            let info =
                CandidateInfoOf::<T>::get(&candidate).ok_or_else(|| Error::<T>::NotCandidate)?;
            Self::add_member(
                &candidate,
                MemberInfo {
                    role: MemberRole::Ally,
                    candidacy: Some(info.candidacy),
                },
            )?;
            CandidateInfoOf::<T>::remove(&candidate);

            Self::deposit_event(Event::CandidateApproved(proposor, candidate));
            Ok(())
        }

        /// Reject a `Candidate` to be a `Outsider`.
        /// Only the members (`Fellows` and `Founders`) can vote to approve/reject the `Candidate`.
        #[pallet::weight(0)]
        pub(crate) fn reject_candidate(
            origin: OriginFor<T>,
            candidate: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let candidate = T::Lookup::lookup(candidate)?;
            ensure!(Self::is_candidate(&candidate), Error::<T>::NotCandidate);
            ensure!(
                !Self::is_suspended(&candidate),
                Error::<T>::AlreadySuspended
            );
            ensure!(!Self::is_member(&candidate), Error::<T>::AlreadyMember);

            let info =
                CandidateInfoOf::<T>::take(&candidate).ok_or_else(|| Error::<T>::NotCandidate)?;
            if let CandidacyKind::Submit(deposit) = info.candidacy {
                T::OnSlash::on_unbalanced(T::Currency::slash_reserved(&candidate, deposit).0);
            }

            Self::deposit_event(Event::CandidateRejected(proposor, candidate));
            Ok(())
        }

        /// Elevate a `Ally` to be a `Fellow`.
        ///
        /// Only the fellows and founders can vote to approve/reject the elevation of the ally.
        /// And the founders has a super right to veto the elevation of ally.
        #[pallet::weight(0)]
        pub(crate) fn elevate_ally(
            origin: OriginFor<T>,
            ally: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let who = T::Lookup::lookup(ally)?;
            ensure!(Self::is_ally_member(&who), Error::<T>::NotAllyMember);
            ensure!(
                !Self::is_votale_member(&who),
                Error::<T>::AlreadyElevatedMember
            );
            ensure!(!Self::is_suspended(&who), Error::<T>::AlreadySuspended);
            ensure!(!Self::is_blacklist(&who), Error::<T>::AlreadyInBlacklist);

            Self::change_member_role(&who, MemberRole::Ally, MemberRole::Fellow)?;

            Self::deposit_event(Event::AllyElevated(proposor, who));
            Ok(())
        }

        /// Retire as a `Member` and back to `Outsider` and unlock deposit (if it is not being kicked)
        #[pallet::weight(0)]
        pub(crate) fn retire(origin: OriginFor<T>) -> DispatchResult {
            let member = ensure_signed(origin)?;
            ensure!(Self::is_member(&member), Error::<T>::NotMember);
            ensure!(!Self::is_suspended(&member), Error::<T>::AlreadySuspended);

            let role = Self::member_role_of(&member).ok_or_else(|| Error::<T>::NotMember)?;
            let info = Self::remove_member(&member, role)?;
            if let Some(CandidacyKind::Submit(deposit)) = info.candidacy {
                T::Currency::unreserve(&member, deposit);
            }

            Self::deposit_event(Event::MemberRetired(member, info.candidacy));
            Ok(())
        }

        /// Kick a `Member` to `Outsider` with its deposit slashed.
        #[pallet::weight(0)]
        pub(crate) fn kick_member(
            origin: OriginFor<T>,
            member: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let member = T::Lookup::lookup(member)?;
            ensure!(Self::is_member(&member), Error::<T>::NotMember);
            ensure!(!Self::is_suspended(&member), Error::<T>::AlreadySuspended);

            let role = Self::member_role_of(&member).ok_or_else(|| Error::<T>::NotMember)?;
            let info = Self::remove_member(&member, role)?;
            match info.candidacy {
                Some(CandidacyKind::Submit(deposit)) => {
                    T::OnSlash::on_unbalanced(T::Currency::slash_reserved(&member, deposit).0);
                }
                Some(CandidacyKind::Nominate(_)) | None => {}
            }

            Self::deposit_event(Event::MemberKicked(proposor, member, info.candidacy));
            Ok(())
        }

        #[pallet::weight(0)]
        pub(crate) fn add_blacklist(
            origin: OriginFor<T>,
            inputs: Vec<<T::Lookup as StaticLookup>::Source>,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let mut accounts = Vec::with_capacity(inputs.len());
            for input in inputs {
                let who = T::Lookup::lookup(input)?;
                ensure!(!Self::is_blacklist(&who), Error::<T>::AlreadyInBlacklist);
                ensure!(!Self::is_suspended(&who), Error::<T>::AlreadySuspended);
                accounts.push(who);
            }

            Self::do_add_blacklist(T::BlockNumber::max_value(), accounts.clone())?;

            Self::deposit_event(Event::BlacklistAdded(accounts));
            Ok(())
        }

        #[pallet::weight(0)]
        pub(crate) fn remove_blacklist(
            origin: OriginFor<T>,
            inputs: Vec<<T::Lookup as StaticLookup>::Source>,
        ) -> DispatchResult {
            let proposor = T::MajorityOrigin::ensure_origin(origin)?;
            ensure!(!Self::is_suspended(&proposor), Error::<T>::AlreadySuspended);

            let mut accounts = Vec::with_capacity(inputs.len());
            for input in inputs {
                let who = T::Lookup::lookup(input)?;
                ensure!(Self::is_blacklist(&who), Error::<T>::NotInBlacklist);
                ensure!(!Self::is_suspended(&who), Error::<T>::AlreadySuspended);
                accounts.push(who);
            }

            Self::do_remove_blacklist(T::BlockNumber::max_value(), &accounts)?;

            Self::deposit_event(Event::BlacklistRemoved(accounts));
            Ok(())
        }
    }
}

// Some helper functions
impl<T: Config> Pallet<T> {
    fn is_candidate(who: &T::AccountId) -> bool {
        CandidateInfoOf::<T>::contains_key(who)
    }

    fn member_role_of(who: &T::AccountId) -> Option<MemberRole> {
        MemberInfoOf::<T>::get(who).map(|info| info.role)
    }

    fn has_member(role: MemberRole) -> bool {
        !Members::<T>::get(role).is_empty()
    }

    fn is_founder(who: &T::AccountId) -> bool {
        Members::<T>::get(MemberRole::Founder).contains(who)
    }

    fn is_member(who: &T::AccountId) -> bool {
        MemberInfoOf::<T>::contains_key(who)
    }

    fn is_votale_member(who: &T::AccountId) -> bool {
        if let Some(info) = MemberInfoOf::<T>::get(who) {
            info.role == MemberRole::Founder || info.role == MemberRole::Fellow
        } else {
            false
        }
    }

    fn is_ally_member(who: &T::AccountId) -> bool {
        if let Some(info) = MemberInfoOf::<T>::get(who) {
            info.role == MemberRole::Ally
        } else {
            false
        }
    }

    fn is_suspended(who: &T::AccountId) -> bool {
        Suspended::<T>::contains_key(who)
    }

    fn is_blacklist(who: &T::AccountId) -> bool {
        Blacklist::<T>::get().contains(who)
    }

    fn change_member_role(who: &T::AccountId, from: MemberRole, to: MemberRole) -> DispatchResult {
        let mut info = Self::remove_member(who, from)?;
        info.role = to;
        Self::add_member(who, info)?;
        Ok(())
    }

    fn add_member(who: &T::AccountId, info: MemberInfo<T>) -> DispatchResult {
        <Members<T>>::mutate(info.role, |members| -> DispatchResult {
            match members.binary_search(who) {
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                Err(pos) => {
                    members.insert(pos, who.clone());
                    if info.role == MemberRole::Founder || info.role == MemberRole::Fellow {
                        T::MembershipChanged::change_members_sorted(&[who.clone()], &[], members);
                    }
                    Ok(())
                }
            }
        })?;
        <MemberInfoOf<T>>::insert(who, info);
        Ok(())
    }

    fn remove_member(who: &T::AccountId, role: MemberRole) -> Result<MemberInfo<T>, DispatchError> {
        <Members<T>>::mutate(role, |members| -> DispatchResult {
            match members.binary_search(who) {
                Ok(pos) => {
                    members.remove(pos);
                    if role == MemberRole::Founder || role == MemberRole::Fellow {
                        T::MembershipChanged::change_members_sorted(&[], &[who.clone()], members);
                    }
                    Ok(())
                }
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        })?;
        let info = <MemberInfoOf<T>>::take(who).ok_or_else(|| Error::<T>::NotMember)?;
        Ok(info)
    }

    fn do_add_blacklist(number: T::BlockNumber, who: Vec<T::AccountId>) -> DispatchResult {
        // let mut blacklist = <Blacklist<T>>::get();
        // let pos = blacklist.binary_search();
        // blacklist.insert(pos, (number, who));
        // <Blacklist<T>>::put(blacklist);
        Ok(())
    }

    fn do_remove_blacklist(number: T::BlockNumber, who: &[T::AccountId]) -> DispatchResult {
        // let mut blacklist = <Blacklist<T>>::get();
        // let pos = blacklist.binary_search();
        // blacklist.remove(pos);
        // <Blacklist<T>>::put(blacklist);
        Ok(())
    }
}

/*
/// Simple ensure origin struct to filter for the alliance (founder + fellow) account.
pub struct EnsureMajority<T>(PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureMajority<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into().and_then(|o| {
            match (
                o,
                Members::<T>::get(MemberRole::Founder),
                Members::<T>::get(MemberRole::Fellow),
            ) {
                (RawOrigin::Signed(ref who), ref founders, ref fellows)
                    if founders.contains(who) || fellows.contains(who) =>
                {
                    Ok(who.clone())
                }
                (r, _, _) => Err(T::Origin::from(r)),
            }
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        let founder = Members::<T>::get(MemberRole::Founder)
            .first()
            .expect("alliance founder should exist")
            .clone();
        T::Origin::from(RawOrigin::Signed(founder))
    }
}

/// Simple ensure origin struct to filter for the alliance member (founder + fellow + ally) account.
pub struct EnsureMember<T>(PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureMember<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into().and_then(|o| {
            match (
                o,
                Members::<T>::get(MemberRole::Founder),
                Members::<T>::get(MemberRole::Fellow),
                Members::<T>::get(MemberRole::Ally),
            ) {
                (RawOrigin::Signed(ref who), ref founders, ref fellows, ref allies)
                    if founders.contains(who) || fellows.contains(who) || allies.contains(who) =>
                {
                    Ok(who.clone())
                }
                (r, _, _, _) => Err(T::Origin::from(r)),
            }
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        let founder = Members::<T>::get(MemberRole::Founder)
            .first()
            .expect("alliance founder should exist")
            .clone();
        T::Origin::from(RawOrigin::Signed(founder))
    }
}

/// Simple ensure origin struct to filter for the alliance founder account.
pub struct EnsureFounder<T>(PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureFounder<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into()
            .and_then(|o| match (o, Members::<T>::get(MemberRole::Founder)) {
                (RawOrigin::Signed(ref who), ref founders) if founders.contains(who) => {
                    Ok(who.clone())
                }
                (r, _) => Err(T::Origin::from(r)),
            })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        let founder = Members::<T>::get(MemberRole::Founder)
            .first()
            .expect("alliance founder should exist")
            .clone();
        T::Origin::from(RawOrigin::Signed(founder))
    }
}

/// Simple ensure origin struct to filter for the alliance fellow account.
pub struct EnsureFellow<T>(PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureFellow<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into()
            .and_then(|o| match (o, Members::<T>::get(MemberRole::Fellow)) {
                (RawOrigin::Signed(ref who), ref fellows) if fellows.contains(who) => {
                    Ok(who.clone())
                }
                (r, _) => Err(T::Origin::from(r)),
            })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        let fellow = Members::<T>::get(MemberRole::Fellow)
            .first()
            .expect("alliance fellow should exist")
            .clone();
        T::Origin::from(RawOrigin::Signed(fellow))
    }
}

/// Simple ensure origin struct to filter for the alliance ally account.
pub struct EnsureAlly<T>(PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureAlly<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into()
            .and_then(|o| match (o, Members::<T>::get(MemberRole::Ally)) {
                (RawOrigin::Signed(ref who), ref allies) if allies.contains(who) => Ok(who.clone()),
                (r, _) => Err(T::Origin::from(r)),
            })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        let ally = Members::<T>::get(MemberRole::Ally)
            .first()
            .expect("alliance ally should exist")
            .clone();
        T::Origin::from(RawOrigin::Signed(ally))
    }
}
*/
