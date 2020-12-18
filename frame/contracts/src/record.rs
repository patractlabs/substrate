use crate::Gas;
use sp_std::cmp::max;
use sp_std::fmt::{self, Formatter};
use sp_std::collections::btree_map::BTreeMap;
use derivative::Derivative;
use crate::env_trace::{EnvTrace, HexVec};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NestedRuntime {
    #[derivative(Debug = "ignore")]
    depth: usize,
    caller: HexVec,
    self_account: Option<HexVec>,
    selector: Option<HexVec>,
    args: Option<HexVec>,
    value: u128,
    gas_limit: Gas,
    gas_left: Gas,
    env_trace: Vec<EnvTrace>,
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
            env_trace: Vec::new(),
        }
    }
}

struct NestedRuntimeWrapper(NestedRuntime);

impl fmt::Debug for NestedRuntimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:#?}", self.0.depth, self.0)
    }
}

#[derive(Debug, Default)]
pub struct Record {
    deepest: usize,
    runtime: Vec<NestedRuntimeWrapper>,
    host_func_count: BTreeMap<String, u8>,
}

impl Record {
    pub fn nested(&mut self, runtime: NestedRuntime) {
        self.deepest = max(self.deepest, runtime.depth);
        self.runtime.push(NestedRuntimeWrapper(runtime));
    }

    pub fn set_gas_left(&mut self, left: Gas, depth: usize) {
        self.runtime
            .get_mut(depth)
            .expect("After `nested`, the index should be exist")
            .0.gas_left = left;
    }

    pub fn set_self_account(&mut self, self_account: Option<HexVec>) {
        self.runtime
            .last_mut()
            .expect("After instantiate, Record::runtime shouldn't be empty")
            .0.self_account = self_account;
    }

    pub fn count(&mut self, host_func: &str) {
        if let Some(count) = self.host_func_count.get_mut(&host_func.to_string()) {
            *count = count.checked_add(1).unwrap();
        } else {
            self.host_func_count.insert(host_func.to_string(), 1);
        }
    }

    fn get_env_trace(&mut self, depth: usize) -> &mut Vec<EnvTrace> {
        &mut self.runtime
            .get_mut(depth - 1)
            .expect("After `nested`, the index should be exist")
            .0.env_trace
    }

    pub fn env_trace_push(&mut self, depth: usize, host_func: EnvTrace) -> usize {
        let env_trace = self.get_env_trace(depth);
        let index = env_trace.len();
        env_trace.push(host_func);
        index
    }

    pub fn env_trace_backtrace(&mut self, depth: usize, index: usize) -> &mut EnvTrace {
        let env_trace = self.get_env_trace(depth);
        env_trace
            .get_mut(index)
            .expect("`update_env_trace` should be used after `env_trace_push`")
    }
}

impl Drop for Record {
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
