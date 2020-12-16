use crate::Gas;
use sp_core::hexdisplay::HexDisplay;
use sp_std::cmp::max;
use sp_std::fmt;
use sp_std::fmt::Formatter;
use sp_std::collections::btree_map::BTreeMap;

pub struct HexVec(Vec<u8>);

impl fmt::Debug for HexVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{}", &HexDisplay::from(&self.0)))
    }
}

impl From<Vec<u8>> for HexVec {
    fn from(vec: Vec<u8>) -> Self {
        HexVec(vec)
    }
}

#[derive(Debug)]
pub struct NestedRuntime {
    depth: usize,
    caller: HexVec,
    self_account: Option<HexVec>,
    selector: Option<HexVec>,
    args: Option<HexVec>,
    value: u128,
    gas_limit: Gas,
    gas_left: Gas,
    seal_trace: Vec<String>,
    // trap_reason: Option<TrapReason>,
}

impl NestedRuntime {
    pub(crate) fn new(
        depth: usize,
        caller: HexVec,
        self_account: Option<HexVec>,
        selector: Option<HexVec>,
        args: Option<HexVec>,
        value: u128,
        gas_limit: Gas,
    ) -> NestedRuntime {
        NestedRuntime {
            depth,
            caller,
            self_account,
            selector,
            args,
            value,
            gas_limit,
            gas_left: gas_limit,
            seal_trace: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Record {
    deepest: usize,
    runtime: Vec<NestedRuntime>,
    seal_count: BTreeMap<String, u8>,
}

impl Record {
    pub(crate) fn new() -> Record {
        Record {
            deepest: 0,
            runtime: vec![],
            seal_count: BTreeMap::new(),
        }
    }

    pub fn nested(&mut self, runtime: NestedRuntime) {
        self.deepest = max(self.deepest, runtime.depth);
        self.runtime.push(runtime);
    }

    pub fn set_gas_left(&mut self, left: Gas, depth: usize) {
        self.runtime
            .get_mut(depth)
            .expect("After `nested`, the index should be exist")
            .gas_left = left;
    }

    pub fn set_self_account(&mut self, self_account: Option<HexVec>) {
        self.runtime
            .last_mut()
            .expect("After instantiate, Record::runtime shouldn't be empty")
            .self_account = self_account;
    }

    pub fn update_seal_trace(&mut self, host_func: &str, depth: usize) {
        self.runtime
            .get_mut(depth)
            .expect("After `nested`, the index should be exist")
            .seal_trace.push(host_func.to_string());
        if let Some(count) = self.seal_count.get_mut(&host_func.to_string()){
            *count = count.checked_add(1).unwrap();
        } else{
            self.seal_count.insert(host_func.to_string(), 1);
        }
    }
}

impl Drop for Record{
    fn drop(&mut self) {
        println! {"{:#?}\n", self};
    }
}

environmental::environmental!(record: Record);

pub fn set_and_run_with_record<F, R>(record: &mut Record, f: F) -> R
    where
        F: FnOnce() -> R,
{
    record::using(record, f)
}

pub fn with_record<F: FnOnce(&mut Record) -> R, R>(f: F) -> Option<R> {
    record::with(f)
}
