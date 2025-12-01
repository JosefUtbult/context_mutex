/// A ContextInterface is a static trait that can retrieve the current running context level. Here
/// is an example from an STM32H743 ARM cortex-m processor
///
/// ```no_run
/// use context_mutex::ContextInterface;
///
/// #[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// enum Level {
///     Interrupt,
///     Kernel,
///     High,
///     Low,
///     Idle
/// }
///
/// impl PartialEq<usize> for Level {
///     fn eq(&self, other: &usize) -> bool {
///         *self as usize == *other
///     }
/// }
///
///
/// struct MyContextInterface {}
/// impl ContextInterface<Level> for MyContextInterface {
///     fn get_current_level() -> Level {
///         // Read the ispr register to get the current level
///         let ipsr: u32;
///         unsafe { core::arch::asm!("mrs {}, IPSR", out(reg) ipsr) };
///
///         // Map the ispr level to an interrupt
///         match ipsr {
///             val if val == 16 + 28 => Level::Kernel,   // TIM2 IRQ
///             val if val == 16 + 29 => Level::High,     // TIM3 IRQ
///             val if val == 16 + 24 => Level::Low,      // TIM4 IRQ
///             val if val == 0 => Level::Idle,           // Thread mode
///             _ => Level::Interrupt                     // Some other interrupt
///         }
///     }
/// }
/// ````
pub trait ContextInterface<ContextType> {
    fn get_current_level() -> ContextType;
}
