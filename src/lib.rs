use core::{cell::UnsafeCell, fmt, marker::PhantomData};

/// A context mutex is a mutex type that relies on a target being compiled for a single core
/// processor, where only one context of each execution level is running at the time. The mutex
/// security comes from that only a single thread can be executed at the same time, but
/// interruptions can result in different context levels. This mutex only allows access during a
/// single execution level.
///
/// **Note that this mutex type isn't safe/viable for multi-thread systems or
/// operating systems that can yield in a context. It is mainly designed for use on
/// single core systems with interrupts.**
///
/// To create a context mutex, you will need a context type, preferably an enum with the different
/// levels that is represented as an usize. The following is an example:
///
/// ```
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
/// ````
///
/// You will also need an Interface type with the `ContextInterface` trait. This needs to be able to
/// report back the current context from a static function. Here is an example from an STM32H743
/// ARM cortex-m processor:
///
/// ```no_run
/// use context_mutex::{Mutex, ContextInterface};
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
/// struct ContextHandler {}
/// impl ContextInterface<Level> for ContextHandler {
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

pub struct Mutex<Interface, Data, ContextType, const LEVEL: usize>
where
    ContextType: fmt::Debug + PartialEq<usize>,
    Interface: ContextInterface<ContextType>,
{
    data: UnsafeCell<Data>,
    _interface: PhantomData<Interface>,
    _context: PhantomData<ContextType>,
}

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
/// struct ContextHandler {}
/// impl ContextInterface<Level> for ContextHandler {
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

pub trait ContextInterface<ContextType>
where
    Self: Sized,
{
    fn get_current_level() -> ContextType;
}

impl<Interface, Data, ContextType, const LEVEL: usize> Mutex<Interface, Data, ContextType, LEVEL>
where
    ContextType: fmt::Debug + PartialEq<usize>,
    Interface: ContextInterface<ContextType>,
{
    #[allow(dead_code)]
    pub const fn new(data: Data) -> Self {
        Self {
            data: UnsafeCell::new(data),
            _interface: PhantomData,
            _context: PhantomData,
        }
    }

    #[allow(dead_code)]
    fn lock<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        let current_level = Interface::get_current_level();
        if current_level != LEVEL {
            panic!(
                "Attempted to lock Mutex in level {:?} from level {:?}",
                LEVEL, current_level
            );
        }

        f(unsafe { &*self.data.get() })
    }

    #[allow(dead_code)]
    fn lock_mut<R>(&self, f: impl FnOnce(&mut Data) -> R) -> R {
        let current_level = Interface::get_current_level();
        if current_level != LEVEL {
            panic!(
                "Attempted to lock Mutex in level {:?} from level {:?}",
                LEVEL, current_level
            );
        }

        f(unsafe { &mut *self.data.get() })
    }
}

impl<Interface, Data, ContextType, const LEVEL: usize> general_mutex::Mutex
    for Mutex<Interface, Data, ContextType, LEVEL>
where
    ContextType: fmt::Debug + PartialEq<usize>,
    Interface: ContextInterface<ContextType>,
{
    type Data = Data;

    fn new(data: Self::Data) -> Self {
        Mutex::new(data)
    }

    fn lock<R>(&self, f: impl FnOnce(&Self::Data) -> R) -> R {
        Mutex::lock(&self, f)
    }

    fn lock_mut<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        Mutex::lock_mut(&self, f)
    }
}

#[cfg(test)]
mod test {
    use crate::{ContextInterface, Mutex};
    use general_mutex::setup_tests;

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    enum Level {
        Level0 = 0,
        Level1,
    }

    impl PartialEq<usize> for Level {
        fn eq(&self, other: &usize) -> bool {
            *self as usize == *other
        }
    }

    struct Handler {}
    impl ContextInterface<Level> for Handler {
        fn get_current_level() -> Level {
            Level::Level0
        }
    }

    // As this mutex type isn't suited for multi-thread systems, this causes compilation to fail
    // during testing. To circumvent this, implement a very unsafe Sync for the mutex type
    unsafe impl<Interface, Data, ContextType, const LEVEL: usize> Sync
        for Mutex<Interface, Data, ContextType, LEVEL>
    where
        ContextType: core::fmt::Debug + PartialEq<usize>,
        Interface: ContextInterface<ContextType>,
    {
    }

    const LEVEL0: usize = Level::Level0 as usize;
    const LEVEL1: usize = Level::Level1 as usize;

    setup_tests!(Mutex<Handler, _, Level, LEVEL0>);

    #[test]
    #[should_panic]
    fn try_lock_from_incorrect_level() {
        let mutex: Mutex<Handler, _, Level, LEVEL1> = Mutex::new(0usize);
        mutex.lock(|_| {})
    }

    #[test]
    #[should_panic]
    fn try_mut_lock_from_incorrect_level() {
        let mutex: Mutex<Handler, _, Level, LEVEL1> = Mutex::new(0usize);
        mutex.lock(|_| {})
    }
}
