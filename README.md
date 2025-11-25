# Context Mutex

A context mutex is a mutex type that relies on a target being compiled for a single core
processor, where only one context of each execution level is running at the time. The mutex
security comes from that only a single thread can be executed at the same time, but
interruptions can result in different context levels. This mutex only allows access during a
single execution level.

**Note that this mutex type isn't safe/viable for multi-thread systems or
operating systems that can yield in a context. It is mainly designed for use on
single core systems with interrupts.**

To create a context mutex, you will need a context type, preferably an enum with the different
levels that is represented as an usize. The following is an example:

```rust
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Level {
    Interrupt,
    Kernel,
    High,
    Low,
    Idle
}

impl PartialEq<usize> for Level {
    fn eq(&self, other: &usize) -> bool {
        *self as usize == *other
    }
}
````

You will also need an Interface type with the `ContextInterface` trait. This needs to be able to
report back the current context from a static function. Here is an example from an STM32H743
ARM cortex-m processor.

```rust
struct ContextHandler {}
impl ContextInterface<Level> for ContextHandler {
    fn get_current_level() -> Level {
        // Read the ispr register to get the current level
        let ipsr: u32;
        unsafe { core::arch::asm!("mrs {}, IPSR", out(reg) ipsr) };

        // Map the ispr level to an interrupt
        match ipsr {
            val if val == 16 + 28 => Level::Kernel,   // TIM2 IRQ
            val if val == 16 + 29 => Level::High,     // TIM3 IRQ
            val if val == 16 + 24 => Level::Low,      // TIM4 IRQ
            val if val == 0 => Level::Idle,           // Thread mode
            _ => Level::Interrupt                     // Some other interrupt
        }
    }
}
````
