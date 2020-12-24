use add_getters_setters::AddSetter;
use derivative::Derivative;
use sp_std::fmt;
use sp_std::fmt::Formatter;
use pallet_contracts_proc_macro::{HostDebug, Wrap};
use serde::{Serialize, Deserialize};

use crate::trace_runtime::with_runtime;

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

pub trait Wrapper: Clone {
    fn wrap(&self) -> EnvTrace;
}

pub struct EnvTraceGuard<T: Wrapper> {
    ptr: *const T,
}

impl<T: Wrapper> EnvTraceGuard<T> {
    pub fn new(seal: &T) -> EnvTraceGuard<T> {
        EnvTraceGuard { ptr: seal as *const T }
    }
}

impl<T: Wrapper> Drop for EnvTraceGuard<T> {
    fn drop(&mut self) {
        with_runtime(|r|
            r.env_trace_push(T::wrap(
                unsafe {
                    &(*self.ptr).clone()
                }
            ))
        ).unwrap();
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct Gas {
    #[set]
    amount: Option<u32>,
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

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealTransfer {
    #[set]
    account: Option<HexVec>,
    #[set]
    value: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealCall {
    #[set]
    callee: Option<HexVec>,
    #[set]
    gas: u64,
    #[set]
    value: Option<u128>,
    #[set]
    input: Option<HexVec>,
    #[set]
    output: Option<HexVec>,
}

impl SealCall {
    pub fn new(gas: u64) -> Self {
        SealCall {
            gas,
            ..Default::default()
        }
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealInstantiate {
    #[set]
    code_hash: Option<HexVec>,
    #[set]
    gas: u64,
    #[set]
    value: Option<u128>,
    #[set]
    input: Option<HexVec>,
    #[set]
    address: Option<HexVec>,
    #[set]
    output: Option<HexVec>,
    #[set]
    salt: Option<HexVec>,
}

impl SealInstantiate {
    pub fn new(gas: u64) -> Self {
        SealInstantiate {
            gas,
            ..Default::default()
        }
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealTerminate {
    #[set]
    beneficiary: Option<HexVec>,
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

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealCaller {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealAddress {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealWeightToFee {
    gas: u64,
    #[set]
    out: Option<u128>,
}

impl SealWeightToFee {
    pub fn new(gas: u64) -> Self {
        SealWeightToFee {
            gas,
            ..Default::default()
        }
    }
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealGasLeft {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealBalance {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealValueTransferred {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealRandom {
    #[set]
    subject: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealNow {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealMinimumBalance {
    #[set]
    out: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealTombstoneDeposit {
    #[set]
    out: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealRestoreTo {
    #[set]
    dest: Option<HexVec>,
    #[set]
    code_hash: Option<HexVec>,
    #[set]
    rent_allowance: Option<u128>,
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

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealSetRentAllowance {
    #[set]
    value: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug, Clone, Wrap)]
pub struct SealRentAllowance {
    #[set]
    out: Option<u128>,
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

#[cfg_attr(feature = "std", derive(Derivative))]
#[derivative(Debug)]
pub enum EnvTrace {
    #[derivative(Debug = "transparent")]
    Gas(Gas),
    #[derivative(Debug = "transparent")]
    SealSetStorage(SealSetStorage),
    #[derivative(Debug = "transparent")]
    SealClearStorage(SealClearStorage),
    #[derivative(Debug = "transparent")]
    SealGetStorage(SealGetStorage),
    #[derivative(Debug = "transparent")]
    SealTransfer(SealTransfer),
    #[derivative(Debug = "transparent")]
    SealCall(SealCall),
    #[derivative(Debug = "transparent")]
    SealInstantiate(SealInstantiate),
    #[derivative(Debug = "transparent")]
    SealTerminate(SealTerminate),
    #[derivative(Debug = "transparent")]
    SealInput(SealInput),
    #[derivative(Debug = "transparent")]
    SealReturn(SealReturn),
    #[derivative(Debug = "transparent")]
    SealCaller(SealCaller),
    #[derivative(Debug = "transparent")]
    SealAddress(SealAddress),
    #[derivative(Debug = "transparent")]
    SealWeightToFee(SealWeightToFee),
    #[derivative(Debug = "transparent")]
    SealGasLeft(SealGasLeft),
    #[derivative(Debug = "transparent")]
    SealBalance(SealBalance),
    #[derivative(Debug = "transparent")]
    SealValueTransferred(SealValueTransferred),
    #[derivative(Debug = "transparent")]
    SealRandom(SealRandom),
    #[derivative(Debug = "transparent")]
    SealNow(SealNow),
    #[derivative(Debug = "transparent")]
    SealMinimumBalance(SealMinimumBalance),
    #[derivative(Debug = "transparent")]
    SealTombstoneDeposit(SealTombstoneDeposit),
    #[derivative(Debug = "transparent")]
    SealRestoreTo(SealRestoreTo),
    #[derivative(Debug = "transparent")]
    SealDepositEvent(SealDepositEvent),
    #[derivative(Debug = "transparent")]
    SealSetRentAllowance(SealSetRentAllowance),
    #[derivative(Debug = "transparent")]
    SealRentAllowance(SealRentAllowance),
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
}