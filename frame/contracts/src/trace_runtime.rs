use sp_std::fmt::{self, Formatter};
use sp_sandbox::{Error, ReturnValue};
use codec::{Decode, Encode};
use frame_support::weights::Weight;

use crate::{
    BalanceOf,
    env_trace::{AccountIdOf, EnvTrace, HexVec},
    exec::{ExecResult, ExecError, ErrorOrigin},
    wasm::runtime::TrapReason, Config
};

/// The host function call stack.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvTraceList<C: Config>(pub Vec<EnvTrace<C>>);

impl<C: Config> fmt::Debug for EnvTraceList<C> {
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



#[derive(PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecReturnValueTrace {
	pub flags: u32,
    #[cfg_attr(feature = "std", serde(with="sp_core::bytes"))]
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
				data: value.data.to_vec()
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct NestedRuntime<C: Config> {
	/// Current depth
    depth: u32,
	/// The current contract execute result
	ext_result: ExecResultTrace,
	/// The value in sandbox successful result
    #[serde(with = "sp_sandbox::serde_opt_wasm_returnvalue")]
	sandbox_result_ok: Option<ReturnValue>,
	/// Who call the current contract
    caller: AccountIdOf<C>,
	/// The account of the current contract
    self_account: AccountIdOf<C>,
	/// The input selector
    selector: Option<HexVec>,
	/// The input arguments
    args: Option<HexVec>,
	/// The value in call or the endowment in instantiate
	#[serde(with = "crate::helper::serde_num_str")]
    value: BalanceOf<C>,
	/// The gas limit when this contract is called
    gas_limit: Weight,
	/// The gas left when this contract return
    gas_left: Weight,
	/// The host function call stack
    env_trace: EnvTraceList<C>,
	/// The error in wasm
    wasm_error: Option<Error>,
	/// The trap in host function execution
    trap_reason: Option<TrapReason>,
	/// Nested contract execution context
    nests: Vec<NestedRuntime<C>>,
}

/// Print `Option<T>`, make `Some` transparent.
fn print_option<T: fmt::Debug>(arg: &Option<T>) -> String {
    if let Some(v) = arg {
        format!("{:?}", v)
    } else{
        String::from("None")
    }
}

impl<C: Config> fmt::Debug for NestedRuntime<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct(&format!("{}: NestedRuntime", &self.depth));

		if let Ok(value) = &self.ext_result {
			debug_struct.field("ext_result", &format_args!("[success] {:?}", value));
		} else if let Err(e) = &self.ext_result{
			debug_struct.field("ext_result", &format_args!("[failed] {:?}", e));
		}

		debug_struct.field("caller", &self.caller)
			.field("self_account", &self.self_account)
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

		debug_struct.field("nests", &self.nests);
		debug_struct.finish()

    }
}

impl<C: Config> NestedRuntime<C> {
    pub fn new(
        depth: u32,
        caller: AccountIdOf<C>,
        self_account: AccountIdOf<C>,
        selector: Option<HexVec>,
        args: Option<HexVec>,
        value: BalanceOf<C>,
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
			nests: Vec::new(),
			wasm_error: None,
			trap_reason: None,
        }
    }

    pub fn none() -> Self {
        Self::new(
            0,
            Default::default(),
            Default::default(),
            None,
            None,
            sp_runtime::traits::Zero::zero(),
            0
        )
    }

    fn is_top_level(&self) -> bool {
        self.depth == 1
    }

    pub fn push_nest(&mut self, nest: NestedRuntime<C>) {
        self.nests.push(nest);
    }

    pub fn last_nest_mut(&mut self) -> &mut NestedRuntime<C> {
        self.nests.last_mut().expect("Must not be empty after `nested`")
    }

    pub fn nests_mut(&mut self) -> &mut Vec<NestedRuntime<C>> {
        &mut self.nests
    }

    pub fn set_gas_left(&mut self, left: Weight) {
        self.gas_left = left;
    }

    pub fn env_trace_push(&mut self, host_func: EnvTrace<C>) {
        let env_trace = &mut self.env_trace;
        env_trace.0.push(host_func);
    }

    pub fn modify_env_trace(&mut self) -> &mut EnvTraceList<C> {
        &mut self.env_trace
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

impl<C: Config> Drop for NestedRuntime<C> {
    fn drop(&mut self) {
        if self.depth == 1 {
            println! {"{:#?}\n", self};
        }
    }
}

// environmental::environmental!(runtime: trait NestedRuntimeT);
mod inner_environmental {
    use super::*;
    use environmental::{GlobalInner, thread_local_impl, using, with};
    #[allow(non_camel_case_types)]
    pub struct runtime<C: Config> {
        __phantom: sp_std::marker::PhantomData<C>
    }
    // a hack way to pass the static compile, using a `dyn Trait` for `GlobalInner<T>`
    // and in `::environmental::with` closure, convert `dyn Trait` to specified struct in unsafe part.
    trait NestedRuntimeT { }
    impl<C: Config> NestedRuntimeT for NestedRuntime<C> { }
    thread_local_impl! {
        static GLOBAL: GlobalInner<dyn NestedRuntimeT> = Default::default()
    }
    impl<C: Config> runtime<C> {
        pub fn using<R, F: FnOnce() -> R>(protected: &mut NestedRuntime<C>, f: F) -> R {
            using(&GLOBAL, protected, f)
        }
        pub fn with<R, F: FnOnce(&mut NestedRuntime<C>) -> R>(f: F) -> Option<R> {
            with(&GLOBAL, |x| {
                let x = unsafe {
                    let p = x as *mut dyn NestedRuntimeT as *mut NestedRuntime<C>;
                    &mut *p
                };
                f(x)
            })
        }
    }
}

pub fn set_and_run_with_runtime<C:Config, F, R>(runtime: &mut NestedRuntime<C>, f: F) -> R
    where
        F: FnOnce() -> R,
{
    use inner_environmental::runtime;
    runtime::using(runtime, f)
}

pub fn with_runtime<C:Config, F: FnOnce(&mut NestedRuntime<C>) -> R, R>(f: F) -> Option<R> {
    use inner_environmental::runtime;
    runtime::with(f)
}

/// Execute the given closure within a nested execution context.
pub fn with_nested_runtime<F, R, C: Config>(
    input_data: Vec<u8>,
    dest: AccountIdOf<C>,
    gas_left: Weight,
    value: BalanceOf<C>,
    depth: usize,
    self_account: AccountIdOf<C>,
    f: F,
) -> (R, Option<NestedRuntime<C>>)
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

    let mut nest = NestedRuntime::new(
        depth as u32,
        self_account,
        dest,
        selector,
        args,
        value,
        gas_left,
    );

    if nest.is_top_level() {
        let r = set_and_run_with_runtime::<C, F, R>(&mut nest, f);
        (r, Some(nest))
    } else {
        let r = with_runtime::<C, _, _>(|r| {
            r.push_nest(nest);
            set_and_run_with_runtime(r.last_nest_mut(), f)
        }).expect("Must not be the top frame contract");
        (r, None)
    }
}
