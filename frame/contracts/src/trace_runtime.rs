use sp_std::fmt::{self, Formatter};
use crate::{env_trace::{EnvTrace, HexVec}, Gas};

#[derive(Debug)]
struct NestedRuntime {
    caller: HexVec,
    self_account: Option<HexVec>,
    selector: Option<HexVec>,
    args: Option<HexVec>,
    value: u128,
    gas_limit: Gas,
    gas_left: Gas,
    env_trace: Vec<EnvTrace>,
    nest: Vec<NestedRuntimeWrapper>,
    // trap_reason: Option<TrapReason>,
}

pub struct NestedRuntimeWrapper {
    depth: usize,
    inner: NestedRuntime,
}

impl NestedRuntimeWrapper {
    pub fn new(
        depth: usize,
        caller: HexVec,
        self_account: Option<HexVec>,
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
            },
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

    pub fn set_self_account(&mut self, self_account: Option<HexVec>) {
        self.inner.self_account = self_account;
    }

    pub fn env_trace_push(&mut self, host_func: EnvTrace) {
        let env_trace = &mut self.inner.env_trace;
        env_trace.push(host_func);
    }
}

impl fmt::Debug for NestedRuntimeWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:#?}", self.depth, self.inner)
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
        Some(account.into())
    } else {
        None
    };

    let mut nest = NestedRuntimeWrapper::new(
        depth,
        self_account.into(),
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