// Windows ARM64 stub for the generator crate.
//
// Upstream (https://github.com/Xudong-Huang/generator-rs/issues/53) does not yet
// support aarch64-pc-windows-msvc. The upstream `detail/mod.rs` references this
// file via `cfg_attr`, but the published crate ships no implementation and no PE
// AAPCS64 assembly backend.
//
// This shim provides the exact symbol surface the rest of the crate requires so
// that downstream users that merely link against `generator` (e.g. kiddo, pulled
// in by reverse_geocoder) compile on Windows ARM64. The functions that perform
// actual register context switching (`swap_registers`, `initialize_call_frame`)
// panic if invoked, because no correct assembly implementation exists here.
//
// This is safe for our tree because the only callers in this repo reach `kiddo`
// through `reverse_geocoder`, which uses `kiddo::float::kdtree::KdTree` and
// `SquaredEuclidean` only — neither path instantiates or switches into a
// generator/coroutine context. If a future caller adds a `kiddo`
// `within_unsorted_iter` query (or otherwise creates a `Generator`) on Windows
// ARM64, it will panic at runtime with the message below instead of silently
// corrupting the stack.
//
// Remove this shim once upstream lands native Windows ARM64 support.

use crate::stack::Stack;

const UNSUPPORTED: &str =
    "generator: aarch64-pc-windows-msvc context switching is not implemented; \
     see src-tauri/third_party/generator-rs/src/detail/aarch64_windows.rs";

// Matches the signature used by the aarch64 unix backend.
pub type InitFn = extern "C" fn(usize, *mut usize) -> !;

pub extern "C" fn gen_init(a1: usize, a2: *mut usize) -> ! {
    super::gen::gen_init_impl(a1, a2)
}

/// Mirrors `Registers` from the aarch64 unix backend (x19..x28, fp, lr, sp, d8..d15).
/// The exact layout is irrelevant here because `swap_registers` panics before
/// reading or writing any of these fields.
#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    gpr: [usize; 32],
}

impl Registers {
    pub fn new() -> Registers {
        Registers { gpr: [0; 32] }
    }

    #[inline]
    pub fn prefetch(&self) {
        // No-op on the stub backend; prefetch is purely a perf hint and has no
        // observable behavior when elided.
    }
}

/// # Safety
///
/// Has the same signature as the real assembly-backed `swap_registers`, but
/// never performs a real context switch. Calling it panics.
pub unsafe fn swap_registers(_out_regs: *mut Registers, _in_regs: *const Registers) {
    panic!("{UNSUPPORTED}");
}

pub fn initialize_call_frame(
    _regs: &mut Registers,
    _fptr: InitFn,
    _arg: usize,
    _arg2: *mut usize,
    _stack: &Stack,
) {
    panic!("{UNSUPPORTED}");
}
