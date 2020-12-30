use sp_std::fmt::{self, Formatter};
use sp_core::crypto::AccountId32;
use sp_std::cmp::min;
use sp_sandbox::Error;

use crate::{env_trace::{EnvTrace, HexVec}, Gas, wasm::runtime::TrapReason};

#[derive(Debug)]
struct NestedRuntime {
    caller: AccountId32,
    self_account: Option<AccountId32>,
    selector: Option<HexVec>,
    args: Option<HexVec>,
    value: u128,
    gas_limit: Gas,
    gas_left: Gas,
    env_trace: Vec<EnvTrace>,
    nest: Vec<NestedRuntimeWrapper>,
    trap_reason: Option<TrapReason>,
}

pub struct NestedRuntimeWrapper {
    depth: usize,
    wasm_error: Option<Error>,
    inner: NestedRuntime,
}

impl NestedRuntimeWrapper {
    pub fn new(
        depth: usize,
        caller: AccountId32,
        self_account: Option<AccountId32>,
        selector: Option<HexVec>,
        args: Option<HexVec>,
        value: u128,
        gas_limit: Gas,
    ) -> Self {
        NestedRuntimeWrapper {
            depth,
            inner: NestedRuntime {
                caller,
                self_account,
                selector,
                args,
                value,
                gas_limit,
                gas_left: gas_limit,
                env_trace: Vec::new(),
                nest: Vec::new(),
                trap_reason: None,
            },
            wasm_error: None
        }
    }

    fn is_top_level(&self) -> bool {
        self.depth == 1
    }

    pub fn nested(&mut self, nest: NestedRuntimeWrapper) {
        self.inner.nest.push(nest);
    }

    pub fn nest_pop(&mut self) -> &mut NestedRuntimeWrapper {
        self.inner.nest.last_mut().expect("Must not be empty after `nested`")
    }

    pub fn set_gas_left(&mut self, left: Gas) {
        self.inner.gas_left = left;
    }

    pub fn set_self_account(&mut self, self_account: Vec<u8>) {
        self.inner.self_account = Some(unchecked_into_account_id32(self_account));
    }

    pub fn env_trace_push(&mut self, host_func: EnvTrace) {
        let env_trace = &mut self.inner.env_trace;
        env_trace.push(host_func);
    }

    pub fn set_trap_reason(&mut self, trap_reason: TrapReason) {
        self.inner.trap_reason = Some(trap_reason);
    }

    pub fn set_wasm_error(&mut self, wasm_error: Error) {
        self.wasm_error = Some(wasm_error);
    }
}

impl fmt::Debug for NestedRuntimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(e)= &self.wasm_error {
            write!(f, "{}: {:#?}\n{:?}", self.depth, self.inner, e)
        } else {
            write!(f, "{}: {:#?}", self.depth, self.inner)
        }
    }
}

impl Drop for NestedRuntimeWrapper {
    fn drop(&mut self) {
        if self.depth == 1 {
            println! {"{:#?}\n", self};
        }
    }
}

environmental::environmental!(runtime: NestedRuntimeWrapper);

pub fn set_and_run_with_runtime<F, R>(runtime: &mut NestedRuntimeWrapper, f: F) -> R
    where
        F: FnOnce() -> R,
{
    runtime::using(runtime, f)
}

pub fn with_runtime<F: FnOnce(&mut NestedRuntimeWrapper) -> R, R>(f: F) -> Option<R> {
    runtime::with(f)
}

fn unchecked_into_account_id32(raw_vec: Vec<u8>)-> AccountId32{
    let mut account = [0u8;32];
    let border = min(raw_vec.len(), 32);
    for i in 0..border {
        account[i] = raw_vec[i]
    }

    AccountId32::from(account)
}

pub fn with_nested_runtime<F, R>(
    input_data: Vec<u8>,
    dest: Option<Vec<u8>>,
    gas_left: Gas,
    value: u128,
    depth: usize,
    self_account: Vec<u8>,
    f: F,
) -> R
    where
        F: FnOnce() -> R {
    let (selector, args) = if input_data.len() > 4 {
        (Some(input_data[0..4].to_vec().into()), Some(input_data[4..].to_vec().into()))
    } else if input_data.len() == 4 {
        (Some(input_data[0..4].to_vec().into()), None)
    } else if input_data.len() > 0 {
        (None, Some(input_data.into()))
    } else {
        (None, None)
    };
    let dest = if let Some(account) = dest {
        Some(unchecked_into_account_id32(account))
    } else {
        None
    };

    let mut nest = NestedRuntimeWrapper::new(
        depth,
        unchecked_into_account_id32(self_account),
        dest,
        selector,
        args,
        value,
        gas_left,
    );

    if nest.is_top_level() {
        set_and_run_with_runtime(&mut nest, f)
    } else {
        with_runtime(|r| {
            r.nested(nest);
            set_and_run_with_runtime(r.nest_pop(), f)
        }).unwrap()
    }
}