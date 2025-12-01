use crate::ContextHandlerCallback;

pub struct ContextLock<'a, ContextType>
where
    ContextType: Clone,
{
    pub(super) handler: &'a dyn ContextHandlerCallback<ContextType>,
    pub(super) level: ContextType,
}

impl<'a, ContextType> Drop for ContextLock<'a, ContextType>
where
    ContextType: Clone,
{
    fn drop(&mut self) {
        self.handler.release(self.level.clone());
    }
}
