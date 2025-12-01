use core::{cell::UnsafeCell, fmt, marker::PhantomData, ops::FnOnce, panic};

use crate::ContextInterface;

pub struct Mutex<Interface, Data, ContextType, const LEVEL: usize>
where
    ContextType: fmt::Debug,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
{
    data: UnsafeCell<Data>,
    _interface: PhantomData<Interface>,
    _context: PhantomData<ContextType>,
}

impl<Interface, Data, ContextType, const LEVEL: usize> Mutex<Interface, Data, ContextType, LEVEL>
where
    ContextType: fmt::Debug,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
{
    pub const fn new(data: Data) -> Self {
        Self {
            data: UnsafeCell::new(data),
            _interface: PhantomData,
            _context: PhantomData,
        }
    }

    pub fn lock<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        let current_level = Interface::get_current_level().into();
        if LEVEL != current_level {
            panic!(
                "Attempted to lock Mutex in level {:?} from level {:?}",
                LEVEL, current_level
            );
        }

        f(unsafe { &*self.data.get() })
    }

    pub fn lock_mut<R>(&self, f: impl FnOnce(&mut Data) -> R) -> R {
        let current_level = Interface::get_current_level().into();
        if LEVEL != current_level {
            panic!(
                "Attempted to lock Mutex in level {:?} from level {:?}",
                LEVEL, current_level
            );
        }

        f(unsafe { &mut *self.data.get() })
    }

    pub unsafe fn unsafe_lock<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        f(unsafe { &*self.data.get() })
    }

    pub unsafe fn unsafe_lock_mut<R>(&self, f: impl FnOnce(&mut Data) -> R) -> R {
        f(unsafe { &mut *self.data.get() })
    }
}

impl<Interface, Data, ContextType, const LEVEL: usize> general_mutex::Mutex
    for Mutex<Interface, Data, ContextType, LEVEL>
where
    ContextType: fmt::Debug,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
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

unsafe impl<Interface, Data, ContextType, const LEVEL: usize> Sync
    for Mutex<Interface, Data, ContextType, LEVEL>
where
    ContextType: fmt::Debug,
    Interface: ContextInterface<ContextType>,
    usize: From<ContextType>,
    Data: Sync,
{
}
