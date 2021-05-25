use add_getters_setters::AddSetter;
use derivative::Derivative;
use serde::{Serialize, Deserialize};

use sp_std::fmt;
use sp_std::fmt::Formatter;

use frame_support::{DefaultNoBound, weights::Weight};
use pallet_contracts_proc_macro::{HostDebug, HostDebugWithGeneric, Wrap, WrapWithGeneric};

use crate::{
    BalanceOf, Config, chain_extension::RetVal, exec::RentParams, trace_runtime::with_runtime,
    rent::RentStatus, wasm::ReturnCode
};

pub type AccountIdOf<C> = <C as frame_system::Config>::AccountId;
type BlockNumberOf<C> = <C as frame_system::Config>::BlockNumber;
type MomentOf<C> = <<C as Config>::Time as frame_support::traits::Time>::Moment;

/// The vector that can be printed as "0x1234"
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct HexVec(Vec<u8>);

impl fmt::Debug for HexVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.len() > 0 {
            f.write_fmt(format_args!("{}", String::from("0x") + &*hex::encode(&self.0)))
        } else {
            write!(f, "")
        }
    }
}

impl From<Vec<u8>> for HexVec {
    fn from(vec: Vec<u8>) -> Self {
        HexVec(vec)
    }
}

impl From<&[u8]> for HexVec {
    fn from(vec: &[u8]) -> Self {
        HexVec(vec.to_vec())
    }
}

impl From<sp_core::Bytes> for HexVec {
    fn from(vec: sp_core::Bytes) -> Self {
        HexVec(vec.0)
    }
}

pub trait Wrapper<C: Config>: Clone {
    fn wrap(&self) -> EnvTrace<C>;
}

pub struct EnvTraceGuard<C: Config, T: Wrapper<C>> {
    ptr: *const T,
    _phantom: sp_std::marker::PhantomData<C>,
}

impl<C: Config, T: Wrapper<C>> EnvTraceGuard<C, T> {
    pub fn new(seal: &T) -> EnvTraceGuard<C, T> {
        EnvTraceGuard { ptr: seal as *const T, _phantom: Default::default() }
    }
}

impl<C: Config, T: Wrapper<C>> Drop for EnvTraceGuard<C, T> {
    fn drop(&mut self) {
        with_runtime(|r|
            r.env_trace_push(T::wrap(
                unsafe {
                    &(*self.ptr).clone()
                }
            ))
        );
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct Gas {
    #[set]
    amount: Option<u32>,
}

impl Gas {
    pub fn is_none(&self) -> bool {
        self.amount.is_none()
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealSetStorage {
    #[set]
    key: Option<HexVec>,
    #[set]
    value: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealClearStorage {
    #[set]
    key: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealGetStorage {
    #[set]
    key: Option<HexVec>,
    #[set]
    output: Option<HexVec>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealTransfer<C: Config> {
    #[set]
    account: Option<AccountIdOf<C>>,
    #[set]
    value: Option<BalanceOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealCall<C: Config> {
    #[set]
    callee: Option<AccountIdOf<C>>,
    #[set]
    gas: Weight,
    #[set]
    value: Option<BalanceOf<C>>,
    #[set]
    input: Option<HexVec>,
    #[set]
    output: Option<HexVec>,
    #[set]
    result: Option<ReturnCode>,
}

impl<C: Config> SealCall<C> {
    pub fn new(gas: Weight) -> Self {
        SealCall {
            gas,
            ..Default::default()
        }
    }
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealInstantiate<C: Config> {
    #[set]
    code_hash: Option<HexVec>,
    #[set]
    gas: Weight,
    #[set]
    value: Option<BalanceOf<C>>,
    #[set]
    input: Option<HexVec>,
    #[set]
    address: Option<AccountIdOf<C>>,
    #[set]
    output: Option<HexVec>,
    #[set]
    salt: Option<HexVec>,
    #[set]
    result: Option<ReturnCode>,
}

impl<C: Config> SealInstantiate<C> {
    pub fn new(gas: Weight) -> Self {
        SealInstantiate {
            gas,
            ..Default::default()
        }
    }
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealTerminate<C: Config> {
    #[set]
    beneficiary: Option<AccountIdOf<C>>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealInput {
    #[set]
    buf: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealReturn {
    flags: u32,
    #[set]
    data: Option<HexVec>,
}

impl SealReturn {
    pub fn new(flags: u32) -> Self {
        SealReturn {
            flags,
            ..Default::default()
        }
    }
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealCaller<C: Config> {
    #[set]
    out: Option<AccountIdOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealAddress<C: Config> {
    #[set]
    out: Option<AccountIdOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealWeightToFee<C: Config> {
    gas: Weight,
    #[set]
    out: Option<BalanceOf<C>>,
}

impl<C: Config> SealWeightToFee<C> {
    pub fn new(gas: Weight) -> Self {
        SealWeightToFee {
            gas,
            ..Default::default()
        }
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealGasLeft {
    #[set]
    out: Option<Weight>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealBalance<C: Config> {
    #[set]
    out: Option<BalanceOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealValueTransferred<C: Config> {
    #[set]
    out: Option<BalanceOf<C>>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealRandom {
    #[set]
    subject: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealRandomV1<C: Config> {
    #[set]
    subject: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
    #[set]
    block_number: Option<BlockNumberOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealNow<C: Config> {
    #[set]
    out: Option<MomentOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealMinimumBalance<C: Config> {
    #[set]
    out: Option<BalanceOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealTombstoneDeposit<C: Config> {
    #[set]
    out: Option<BalanceOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealRestoreTo<C: Config> {
    #[set]
    dest: Option<AccountIdOf<C>>,
    #[set]
    code_hash: Option<HexVec>,
    #[set]
    rent_allowance: Option<BalanceOf<C>>,
    #[set]
    delta: Option<Vec<HexVec>>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealDepositEvent {
    #[set]
    topics: Option<Vec<HexVec>>,
    #[set]
    data: Option<HexVec>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealSetRentAllowance<C: Config> {
    #[set]
    value: Option<BalanceOf<C>>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealRentAllowance<C: Config> {
    #[set]
    out: Option<BalanceOf<C>>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealPrintln {
    #[set]
    str: Option<String>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealBlockNumber {
    #[set]
    out: Option<u32>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealHashSha256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealHashKeccak256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealHashBlake256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealHashBlake128 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(frame_support::DefaultNoBound, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealChainExtension {
    #[set]
    func_id: Option<u32>,
    #[set]
    ret_val: Option<RetVal>
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealRentParams<C: Config> {
    #[set]
    params: RentParams<C>,
}

#[derive(DefaultNoBound, AddSetter, Clone, HostDebugWithGeneric, WrapWithGeneric)]
pub struct SealRentStatus<C: Config> {
    #[set]
    params: RentStatus<C>,
}

#[cfg_attr(feature = "std", derive(Derivative))]
#[derivative(Debug(bound="C: Config"))]
pub enum EnvTrace<C: Config> {
    #[derivative(Debug = "transparent")]
    Gas(Gas),
    #[derivative(Debug = "transparent")]
    SealSetStorage(SealSetStorage),
    #[derivative(Debug = "transparent")]
    SealClearStorage(SealClearStorage),
    #[derivative(Debug = "transparent")]
    SealGetStorage(SealGetStorage),
    #[derivative(Debug = "transparent")]
    SealTransfer(SealTransfer<C>),
    #[derivative(Debug = "transparent")]
    SealCall(SealCall<C>),
    #[derivative(Debug = "transparent")]
    SealInstantiate(SealInstantiate<C>),
    #[derivative(Debug = "transparent")]
    SealTerminate(SealTerminate<C>),
    #[derivative(Debug = "transparent")]
    SealInput(SealInput),
    #[derivative(Debug = "transparent")]
    SealReturn(SealReturn),
    #[derivative(Debug = "transparent")]
    SealCaller(SealCaller<C>),
    #[derivative(Debug = "transparent")]
    SealAddress(SealAddress<C>),
    #[derivative(Debug = "transparent")]
    SealWeightToFee(SealWeightToFee<C>),
    #[derivative(Debug = "transparent")]
    SealGasLeft(SealGasLeft),
    #[derivative(Debug = "transparent")]
    SealBalance(SealBalance<C>),
    #[derivative(Debug = "transparent")]
    SealValueTransferred(SealValueTransferred<C>),
    #[derivative(Debug = "transparent")]
    SealRandom(SealRandom),
    #[derivative(Debug = "transparent")]
    SealRandomV1(SealRandomV1<C>),
    #[derivative(Debug = "transparent")]
    SealNow(SealNow<C>),
    #[derivative(Debug = "transparent")]
    SealMinimumBalance(SealMinimumBalance<C>),
    #[derivative(Debug = "transparent")]
    SealTombstoneDeposit(SealTombstoneDeposit<C>),
    #[derivative(Debug = "transparent")]
    SealRestoreTo(SealRestoreTo<C>),
    #[derivative(Debug = "transparent")]
    SealDepositEvent(SealDepositEvent),
    #[derivative(Debug = "transparent")]
    SealSetRentAllowance(SealSetRentAllowance<C>),
    #[derivative(Debug = "transparent")]
    SealRentAllowance(SealRentAllowance<C>),
    #[derivative(Debug = "transparent")]
    SealPrintln(SealPrintln),
    #[derivative(Debug = "transparent")]
    SealBlockNumber(SealBlockNumber),
    #[derivative(Debug = "transparent")]
    SealHashSha256(SealHashSha256),
    #[derivative(Debug = "transparent")]
    SealHashKeccak256(SealHashKeccak256),
    #[derivative(Debug = "transparent")]
    SealHashBlake256(SealHashBlake256),
    #[derivative(Debug = "transparent")]
    SealHashBlake128(SealHashBlake128),
    #[derivative(Debug = "transparent")]
    SealChainExtension(SealChainExtension),
    #[derivative(Debug = "transparent")]
    SealRentParams(SealRentParams<C>),
    #[derivative(Debug = "transparent")]
    SealRentStatus(SealRentStatus<C>),
}
