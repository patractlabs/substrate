use add_getters_setters::AddSetter;
use derivative::Derivative;
use sp_std::fmt;
use sp_std::fmt::Formatter;
use pallet_contracts_proc_macro::HostDebug;
use sp_core::hexdisplay::HexDisplay;
use crate::exec::StorageKey;

pub struct HexVec(Vec<u8>);

impl fmt::Debug for HexVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.len() > 0 {
            f.write_fmt(format_args!("0x{:?}", &HexDisplay::from(&self.0)))
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

#[derive(Default, AddSetter, HostDebug)]
pub struct Gas {
    #[set]
    amount: Option<u32>
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealSetStorage {
    #[set]
    key: Option<StorageKey>,
    #[set]
    value: Option<Vec<u8>>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealClearStorage {
    #[set]
    key: Option<StorageKey>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealGetStorage {
    #[set]
    key: Option<StorageKey>,
    #[set]
    output: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealTransfer {
    #[set]
    account: Option<HexVec>,
    #[set]
    value: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealCall {
    #[set]
    callee: Option<HexVec>,
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

#[derive(Default, AddSetter, HostDebug)]
pub struct SealInstantiate {
    #[set]
    code_hash: Option<HexVec>,
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

#[derive(Default, AddSetter, HostDebug)]
pub struct SealTerminate {
    #[set]
    beneficiary: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealInput {
    #[set]
    buf: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
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

#[derive(Default, AddSetter, HostDebug)]
pub struct SealCaller {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealAddress {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
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

#[derive(Default, AddSetter, HostDebug)]
pub struct SealGasLeft {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealBalance {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealValueTransferred {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealRandom {
    #[set]
    subject: Option<HexVec>,
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealNow {
    #[set]
    out: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealMinimumBalance {
    #[set]
    out: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealTombstoneDeposit {
    #[set]
    out: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealRestoreTo {
    #[set]
    dest: Option<HexVec>,
    #[set]
    code_hash: Option<HexVec>,
    #[set]
    rent_allowance: Option<u128>,
    #[set]
    delta: Option<Vec<StorageKey>>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealDepositEvent {
    #[set]
    topics: Option<Vec<HexVec>>,
    #[set]
    data: Option<HexVec>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealSetRentAllowance {
    #[set]
    value: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealRentAllowance {
    #[set]
    out: Option<u128>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealPrintln {
    #[set]
    str: Option<String>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealBlockNumber {
    #[set]
    out: Option<u32>,
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealHashSha256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealHashKeccak256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealHashBlake256 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>
}

#[derive(Default, AddSetter, HostDebug)]
pub struct SealHashBlake128 {
    #[set]
    input: Option<HexVec>,
    #[set]
    out: Option<HexVec>
}

#[derive(Derivative)]
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