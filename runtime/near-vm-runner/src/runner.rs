use near_primitives::types::CompiledContractCache;
use near_runtime_fees::RuntimeFeesConfig;
use near_vm_errors::VMError;
use near_vm_logic::types::{ProfileData, PromiseResult, ProtocolVersion};
use near_vm_logic::{External, VMConfig, VMContext, VMKind, VMOutcome};

/// `run` does the following:
/// - deserializes and validate the `code` binary (see `prepare::prepare_contract`)
/// - injects gas counting into
/// - adds fee to VMLogic's GasCounter for size of contract
/// - instantiates (links) `VMLogic` externs with the imports of the binary
/// - calls the `method_name` with `context.input`
///   - updates `ext` with new receipts, created during the execution
///   - counts burnt and used gas
///   - counts how accounts storage usage increased by the call
///   - collects logs
///   - sets the return data
///  returns result as `VMOutcome`
pub fn run<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
    profile: &Option<ProfileData>,
) -> (Option<VMOutcome>, Option<VMError>) {
    match profile {
        Some(profile) => run_vm_profiled(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            VMKind::default(),
            profile.clone(),
            current_protocol_version,
            cache,
        ),
        _ => run_vm(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            VMKind::default(),
            current_protocol_version,
            cache,
        ),
    }
}
pub fn run_vm<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    vm_kind: VMKind,
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
) -> (Option<VMOutcome>, Option<VMError>) {
    use crate::wasmer_runner::run_wasmer;
    #[cfg(feature = "wasmtime_vm")]
    use crate::wasmtime_runner::wasmtime_runner::run_wasmtime;
    match vm_kind {
        VMKind::Wasmer => run_wasmer(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            None,
            current_protocol_version,
            cache,
        ),
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => run_wasmtime(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            None,
            current_protocol_version,
            cache,
        ),
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    }
}

pub fn run_vm_profiled<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    vm_kind: VMKind,
    profile: ProfileData,
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
) -> (Option<VMOutcome>, Option<VMError>) {
    use crate::wasmer_runner::run_wasmer;
    #[cfg(feature = "wasmtime_vm")]
    use crate::wasmtime_runner::wasmtime_runner::run_wasmtime;
    let (outcome, error) = match vm_kind {
        VMKind::Wasmer => run_wasmer(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            Some(profile.clone()),
            current_protocol_version,
            cache,
        ),
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => run_wasmtime(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            Some(profile.clone()),
            current_protocol_version,
            cache,
        ),
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    };
    match &outcome {
        Some(VMOutcome { burnt_gas, .. }) => profile.set_burnt_gas(*burnt_gas),
        _ => (),
    };
    (outcome, error)
}

pub fn with_vm_variants(runner: fn(VMKind) -> ()) {
    runner(VMKind::Wasmer);
    #[cfg(feature = "wasmtime_vm")]
    runner(VMKind::Wasmtime);
}

/// Used for testing cost of compiling a module
pub fn compile_module(vm_kind: VMKind, code: &Vec<u8>) -> bool {
    match vm_kind {
        VMKind::Wasmer => {
            use crate::wasmer_runner::compile_module;
            compile_module(code)
        }
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => {
            use crate::wasmtime_runner::compile_module;
            compile_module(code)
        }
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    };
    false
}
