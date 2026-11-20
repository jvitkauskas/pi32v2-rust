# `abi_pi32v2_interrupt`

The tracking issue for this feature is: None.

------------------------

On JieLi's pi32v2 (`pi32v2-unknown-none`), interrupt handlers return with `rti` and must leave every register as they
found it. The `"pi32v2-interrupt"` ABI gives a handler that convention, as clang's `__attribute__((interrupt))` does:
it saves `psr` and `reti`, and every register it uses (and, if it calls anything, the ones the call may change), and
returns with `rti`. A handler takes no arguments and returns `()` or `!`, and can't be called from Rust.

``` rust,ignore (pi32v2 only)
#![feature(abi_pi32v2_interrupt)]
#![no_std]

static mut TICKS: u32 = 0;

extern "pi32v2-interrupt" fn timer5() {
    unsafe { TICKS += 1 };
    // acknowledge the timer's request here
}

// Its address goes in the interrupt vector table, for example:
// unsafe { (0x01C7FE00 as *mut u32).add(63).write_volatile(timer5 as usize as u32) }
```
