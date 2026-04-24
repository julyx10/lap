use super::windows_bindings::Windows::Win32::{
    Foundation::EXCEPTION_STACK_OVERFLOW,
    System::Diagnostics::Debug::{AddVectoredExceptionHandler, CONTEXT, EXCEPTION_POINTERS},
};
#[cfg(target_arch = "x86_64")]
use crate::rt::{guard, Context, ContextStack};
#[cfg(target_arch = "x86_64")]
use std::sync::Once;

// Stack overflow detection uses x86_64 CONTEXT register names (Rsp, Rbx, ...)
// that don't exist on aarch64 Windows. The aarch64 backend in
// `detail/aarch64_windows.rs` panics before any coroutine is actually entered,
// so we don't need overflow detection there — install a no-op init on aarch64.

#[cfg(target_arch = "x86_64")]
unsafe extern "system" fn vectored_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;

    let info = &*exception_info;
    let rec = &(*info.ExceptionRecord);
    let context = &mut (*info.ContextRecord);

    if rec.ExceptionCode == EXCEPTION_STACK_OVERFLOW
        && guard::current().contains(&(context.Rsp as usize))
    {
        eprintln!(
            "\ncoroutine in thread '{}' has overflowed its stack\n",
            std::thread::current().name().unwrap_or("<unknown>")
        );

        let env = ContextStack::current();
        let cur = env.top();
        cur.err = Some(Box::new(crate::Error::StackErr));

        context_init(env.pop_context(cur as *mut _), context);

        //yield_now();

        EXCEPTION_CONTINUE_EXECUTION
    } else {
        EXCEPTION_CONTINUE_SEARCH
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn init() {
    AddVectoredExceptionHandler(1, Some(vectored_handler));
}

#[cfg(target_arch = "x86_64")]
pub fn init_once() {
    static INIT_ONCE: Once = Once::new();

    INIT_ONCE.call_once(|| unsafe {
        init();
    })
}

// aarch64-pc-windows-msvc: stack overflow detection is skipped because the
// aarch64_windows detail backend panics rather than performing a real context
// switch, so there is no coroutine stack to guard.
#[cfg(not(target_arch = "x86_64"))]
#[allow(dead_code)]
fn _keep_imports_alive() {
    let _ = AddVectoredExceptionHandler;
    let _: Option<&CONTEXT> = None;
    let _: Option<&EXCEPTION_POINTERS> = None;
    let _ = EXCEPTION_STACK_OVERFLOW;
}

#[cfg(not(target_arch = "x86_64"))]
pub fn init_once() {}

#[cfg(target_arch = "x86_64")]
unsafe fn context_init(parent: &mut Context, context: &mut CONTEXT) {
    let [rbx, rsp, rbp, _, r12, r13, r14, r15, _, _, _, stack_base, stack_limit, dealloc_stack, ..] =
        parent.regs.regs.gpr;

    let rip = *(rsp as *const usize);
    let rsp = rsp + std::mem::size_of::<usize>();

    context.Rbx = rbx as u64;
    context.Rsp = rsp as u64;
    context.Rbp = rbp as u64;
    context.R12 = r12 as u64;
    context.R13 = r13 as u64;
    context.R14 = r14 as u64;
    context.R15 = r15 as u64;
    context.Rip = rip as u64;

    let teb: usize;

    unsafe {
        std::arch::asm!(
        "mov {0}, gs:[0x30]",
        out(reg) teb
        );
    }

    *((teb + 0x08) as *mut usize) = stack_base;
    *((teb + 0x10) as *mut usize) = stack_limit;
    *((teb + 0x1478) as *mut usize) = dealloc_stack;
}
