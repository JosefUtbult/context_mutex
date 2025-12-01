use core::{
    fmt,
    marker::PhantomData,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::{ContextInterface, context_lock::ContextLock};

/// A status is a set of counters containing lock acquiring information
pub struct Status {
    pub successful_attempts: usize,
    pub failed_attempts: usize,
}

struct Level {
    is_running: AtomicBool,
    successful_attempts: AtomicUsize,
    failed_attempts: AtomicUsize,
}

// This struct keeps track of the number of times contexts are entered and exited. If a context is
// entered by an interrupt before it has ended, it should exit and will be reported to as overrun
pub struct ContextHandler<Interface, ContextType, const LEVEL_COUNT: usize>
where
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
{
    levels: [Level; LEVEL_COUNT],
    _type: PhantomData<ContextType>,
    _interface: PhantomData<Interface>,
}

// Callback trait to simplify templating on context locks
pub(super) trait ContextHandlerCallback<ContextType> {
    fn release(&self, context: ContextType);
}

impl<Interface, ContextType, const LEVEL_COUNT: usize>
    ContextHandler<Interface, ContextType, LEVEL_COUNT>
where
    ContextType: fmt::Debug + Clone,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
{
    /// Create a new context handler. This should preferably be static and accessible by all that
    /// requires usage of context mutex
    pub const fn new() -> Self {
        Self {
            levels: [const {
                Level {
                    is_running: AtomicBool::new(false),
                    successful_attempts: AtomicUsize::new(0),
                    failed_attempts: AtomicUsize::new(0),
                }
            }; LEVEL_COUNT],
            _interface: PhantomData,
            _type: PhantomData,
        }
    }

    /// Retrieves the current context status and clears all counters
    pub fn get_status(&self) -> [Status; LEVEL_COUNT] {
        core::array::from_fn(|index| {
            let current_level = &self.levels[index];

            let successful_attempts = current_level.successful_attempts.swap(0, Ordering::SeqCst);
            let failed_attempts = current_level.failed_attempts.swap(0, Ordering::SeqCst);

            Status {
                successful_attempts,
                failed_attempts,
            }
        })
    }

    /// Take a context lock for the currently active level, if available. Otherwise, returns none.
    /// In this case, a context should stop and return
    pub fn lock<'a>(&'a self) -> Option<ContextLock<'a, ContextType>> {
        let current_level = Interface::get_current_level();
        let current_level_index: usize = current_level.clone().into();
        assert!(current_level_index < LEVEL_COUNT);
        let current_level_ref = &self.levels[current_level_index];

        if current_level_ref.is_running.swap(true, Ordering::SeqCst) {
            current_level_ref
                .failed_attempts
                .fetch_add(1, Ordering::SeqCst);

            None
        } else {
            current_level_ref
                .successful_attempts
                .fetch_add(1, Ordering::SeqCst);

            Some(ContextLock {
                handler: self,
                level: current_level,
            })
        }
    }
}

impl<Interface, ContextType, const LEVEL_COUNT: usize> ContextHandlerCallback<ContextType>
    for ContextHandler<Interface, ContextType, LEVEL_COUNT>
where
    ContextType: Clone,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
{
    /// Release an active context lock. Should be called by the dropping of a context lock
    fn release(&self, level: ContextType) {
        let level: usize = level.into();
        self.levels[level].is_running.store(false, Ordering::SeqCst);
    }
}
