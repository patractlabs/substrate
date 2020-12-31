use sp_std::fmt::{self, Formatter};
use sp_core::crypto::AccountId32;
use sp_std::cmp::min;
use sp_sandbox::Error;

use crate::{env_trace::{EnvTrace, HexVec}, Gas, wasm::runtime::TrapReason};

struct EnvTraceList(Vec<EnvTrace>);

impl fmt::Debug for EnvTraceList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()
            .filter(|t| {
                if let EnvTrace::Gas(g) = t {
                    g.is_none()
                } else {
                    true
                }
            }))
            .finish()
    }
}

#[derive(sp_core::RuntimeDebug)]
struct NestedRuntime {
    caller: AccountId32,
    self_account: Option<AccountId32>,
    selector: Option<HexVec>,
    args: Option<HexVec>,
    value: u128,
    gas_limit: Gas,
    gas_left: Gas,
    env_trace: EnvTraceList,
    nest: Vec<NestedRuntimeWrapper>,
}

pub struct NestedRuntimeWrapper {
    depth: usize,
    wasm_error: Option<Error>,
    trap_reason: Option<TrapReason>,
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
                env_trace: EnvTraceList(Vec::new()),
                nest: Vec::new(),
            },
            wasm_error: None,
            trap_reason: None,
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
        env_trace.0.push(host_func);
    }

    pub fn set_trap_reason(&mut self, trap_reason: TrapReason) {
        self.trap_reason = Some(trap_reason);
    }

    pub fn set_wasm_error(&mut self, wasm_error: Error) {
        self.wasm_error = Some(wasm_error);
    }
}

impl fmt::Debug for NestedRuntimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut arg = format!("{}: {:#?}", self.depth, self.inner);
        if let Some(e)= &self.wasm_error {
            let err_str = format!("\n{:?}", e);
            arg += &err_str;
        }
        if let Some(trap) = &self.trap_reason {
            let trap_str = format!("\n{:?}", trap);
            arg += &trap_str;
        }
        f.write_str(&arg)
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