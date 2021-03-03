use sp_std::fmt::{self, Formatter};
use sp_core::crypto::AccountId32;
use sp_std::cmp::min;
use sp_sandbox::{Error, ReturnValue};
use pallet_contracts_primitives::{ExecResult, ExecError, ErrorOrigin};
use codec::{Decode, Encode};
use frame_support::weights::Weight;

use crate::{env_trace::{EnvTrace, HexVec}, wasm::runtime::TrapReason};

/// The host function call stack.
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

// /// Wrap sandbox::Error for fmt::Debug.
// struct WasmErrorWrapper(Error);
// 
// impl fmt::Debug for WasmErrorWrapper {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		match &self.0 {
// 			Error::WasmiExecution(err) => {
// 				match err {
// 					WasmiError::Trap(trap) => {
// 						write!(f, "Error::WasmiExecution(Trap(Trap {{ kind: {:?} }}))\n", trap.kind())?;
// 						write!(f, "\twasm backtrace: ")?;
// 
// 						for (index, trace) in trap.wasm_trace().iter().enumerate() {
// 							if index == trap.wasm_trace().len() - 1{
// 								write!(f, "\n\t╰─>")?;
// 							} else {
// 								write!(f, "\n\t|  ")?;
// 							}
// 							write!(f, "{}", trace)?;
// 						}
// 
// 						if trap.wasm_trace().is_empty() {
// 							write!(f, "[]")?;
// 						}
// 
// 						write!(f, "\n")
// 					},
// 					_ => {
// 						write!(f, "{:?}", self.0)
// 					}
// 				}
// 			},
// 			error => {
// 				write!(f, "{:?}", error)
// 			}
// 		}
// 	}
// }

#[derive(PartialEq, Eq, Encode, Decode)]
pub struct ExecReturnValueTrace {
	pub flags: u32,
	pub data: Vec<u8>,
}

impl fmt::Debug for ExecReturnValueTrace {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("ExecReturnValue")
			.field("flags", &self.flags)
			.field("data", &format_args!("{}", &hex::encode(&self.data)))
			.finish()
	}
}

pub type ExecResultTrace = Result<ExecReturnValueTrace, ExecError>;

pub fn into_exec_result_trace(ext_result: &ExecResult) -> ExecResultTrace {
	match ext_result {
		Ok(value) => {
			Ok(ExecReturnValueTrace {
				flags: {
					if value.is_success() {
						0
					} else {
						1
					}
				},
				data: value.data.clone()
			})
		},
		Err(e) => {
			Err(ExecError{
				error: e.error.clone(),
				origin: {
					match e.origin {
						ErrorOrigin::Callee => ErrorOrigin::Callee,
						ErrorOrigin::Caller => ErrorOrigin::Caller,
					}
				}
			})
		}
	}
}

/// Record the contract execution context.
pub struct NestedRuntime {
	/// Current depth
    depth: usize,
	/// The current contract execute result
	ext_result: ExecResultTrace,
	/// The value in sandbox successful result
	sandbox_result_ok: Option<ReturnValue>,
	/// Who call the current contract
    caller: AccountId32,
	/// The account of the current contract
    self_account: Option<AccountId32>,
	/// The input selector
    selector: Option<HexVec>,
	/// The input arguments
    args: Option<HexVec>,
	/// The value in call or the endowment in instantiate
    value: u128,
	/// The gas limit when this contract is called
    gas_limit: Weight,
	/// The gas left when this contract return
    gas_left: Weight,
	/// The host function call stack
    env_trace: EnvTraceList,
	/// The error in wasm
    wasm_error: Option<Error>,
	/// The trap in host function execution
    trap_reason: Option<TrapReason>,
	/// Nested contract execution context
    nest: Vec<NestedRuntime>,
}

/// Print `Option<T>`, make `Some` transparent.
fn print_option<T: fmt::Debug>(arg: &Option<T>) -> String {
    if let Some(v) = arg {
        format!("{:?}", v)
    } else{
        String::from("None")
    }
}

impl fmt::Debug for NestedRuntime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct(&format!("{}: NestedRuntime", &self.depth));

		if let Ok(value) = &self.ext_result {
			debug_struct.field("ext_result", &format_args!("[success] {:?}", value));
		} else if let Err(e) = &self.ext_result{
			debug_struct.field("ext_result", &format_args!("[failed] {:?}", e));
		}

		debug_struct.field("caller", &self.caller)
			.field("self_account", &format_args!("{}", print_option(&self.self_account)))
			.field("selector", &format_args!("{}", print_option(&self.selector)))
			.field("args", &format_args!("{}", print_option(&self.args)))
			.field("value", &self.value)
			.field("gas_limit", &self.gas_limit)
			.field("gas_left", &self.gas_left)
			.field("env_trace", &self.env_trace);

		if let Some(sandbox_result) = &self.sandbox_result_ok {
			debug_struct.field("sandbox_result_ok", sandbox_result);
		}

		if let Some(trap) = &self.trap_reason {
			debug_struct.field("trap_reason", &format_args!("{:?}", trap));
		}

		if self.ext_result.is_err() {
			if let Some(wasm_err) = &self.wasm_error {
				debug_struct.field("wasm_error", wasm_err);
			}
		}

		debug_struct.field("nest", &self.nest);
		debug_struct.finish()

    }
}

impl NestedRuntime {
    pub fn new(
        depth: usize,
        caller: AccountId32,
        self_account: Option<AccountId32>,
        selector: Option<HexVec>,
        args: Option<HexVec>,
        value: u128,
        gas_limit: Weight,
    ) -> Self {
        NestedRuntime {
            depth,
			ext_result: Ok(
				ExecReturnValueTrace {
					flags: 0,
					data: Vec::new(),
			}),
			sandbox_result_ok: None,
			caller,
			self_account,
			selector,
			args,
			value,
			gas_limit,
			gas_left: gas_limit,
			env_trace: EnvTraceList(Vec::new()),
			nest: Vec::new(),
			wasm_error: None,
			trap_reason: None,
        }
    }

    fn is_top_level(&self) -> bool {
        self.depth == 1
    }

    pub fn nested(&mut self, nest: NestedRuntime) {
        self.nest.push(nest);
    }

    pub fn nest_pop(&mut self) -> &mut NestedRuntime {
        self.nest.last_mut().expect("Must not be empty after `nested`")
    }

    pub fn set_gas_left(&mut self, left: Weight) {
        self.gas_left = left;
    }

    pub fn set_self_account(&mut self, self_account: Vec<u8>) {
        self.self_account = Some(unchecked_into_account_id32(self_account));
    }

    pub fn env_trace_push(&mut self, host_func: EnvTrace) {
        let env_trace = &mut self.env_trace;
        env_trace.0.push(host_func);
    }

    pub fn set_trap_reason(&mut self, trap_reason: TrapReason) {
        self.trap_reason = Some(trap_reason);
    }

    pub fn set_wasm_error(&mut self, wasm_error: Error) {
        self.wasm_error = Some(wasm_error);
    }

	pub fn set_ext_result(&mut self, ext_result: ExecResultTrace) {
		self.ext_result = ext_result;
	}

	pub fn set_sandbox_result(&mut self, sandbox_result: ReturnValue) {
		self.sandbox_result_ok = Some(sandbox_result);
	}
}

impl Drop for NestedRuntime {
    fn drop(&mut self) {
        if self.depth == 1 {
            println! {"{:#?}\n", self};
        }
    }
}

environmental::environmental!(runtime: NestedRuntime);

pub fn set_and_run_with_runtime<F, R>(runtime: &mut NestedRuntime, f: F) -> R
    where
        F: FnOnce() -> R,
{
    runtime::using(runtime, f)
}

pub fn with_runtime<F: FnOnce(&mut NestedRuntime) -> R, R>(f: F) -> Option<R> {
    runtime::with(f)
}

/// Unchecked convert the account to `AccountId32`.
fn unchecked_into_account_id32(raw_vec: Vec<u8>)-> AccountId32{
    let mut account = [0u8;32];
    let border = min(raw_vec.len(), 32);
    for i in 0..border {
        account[i] = raw_vec[i]
    }

    AccountId32::from(account)
}

/// Execute the given closure within a nested execution context.
pub fn with_nested_runtime<F, R>(
    input_data: Vec<u8>,
    dest: Option<Vec<u8>>,
    gas_left: Weight,
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

    let mut nest = NestedRuntime::new(
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
