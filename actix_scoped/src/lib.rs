use std::cell::OnceCell;
use std::collections::HashMap;
use std::thread::ThreadId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub trait Scoped: HasThreadContext + ScopedDefault {
    fn __get_thread_id() -> ThreadId {
        let thread = std::thread::current();
        thread.id()
    }

    fn initialize() -> ScopedResult<()> {
        let thread_id = Self::__get_thread_id();
        let ctx = Self::__get_context_mut()?;
        ctx.insert(thread_id.clone(), Self::scoped_default());
        Ok(())
    }
    fn get_or_initialize<'a>() -> ScopedResult<&'a Self::Value> {
        let thread_id = Self::__get_thread_id();
        let ctx = Self::__get_context_mut()?;
        if !ctx.contains_key(&thread_id) {
            ctx.insert(thread_id.clone(), Self::scoped_default());
        }

        Ok(ctx.get(&thread_id).unwrap())
    }
    fn get<'a>() -> ScopedResult<Option<&'a Self::Value>> {
        let thread_id = Self::__get_thread_id();
        let ctx = Self::__get_context()?;
        Ok(ctx.get(&thread_id))
    }
    fn set(value: Self::Value) -> ScopedResult<Option<Self::Value>> {
        let thread_id = Self::__get_thread_id();
        let ctx = Self::__get_context_mut()?;
        Ok(ctx.insert(thread_id, value))
    }
    fn clear() -> ScopedResult<()> {
        let thread_id = Self::__get_thread_id();
        let ctx = Self::__get_context_mut()?;
        ctx.remove(&thread_id);
        Ok(())
    }
}

pub trait ScopedDefault: HasThreadContext {
    fn scoped_default() -> Self::Value;
}

pub trait HasThreadContext {
    type Value: Clone;

    unsafe fn thread_context_raw<'a>() -> &'a mut OnceCell<HashMap<ThreadId, Self::Value>>;

    fn __get_context<'a>() -> ScopedResult<&'a HashMap<ThreadId, Self::Value>> {
        unsafe { Self::thread_context_raw().get() }.ok_or(ScopedError::Uninitialized)
    }
    fn __get_context_mut<'a>() -> ScopedResult<&'a mut HashMap<ThreadId, Self::Value>> {
        unsafe { Self::thread_context_raw().get_mut() }.ok_or(ScopedError::Uninitialized)
    }
    fn __init_context(context: HashMap<ThreadId, Self::Value>) -> ScopedResult<()> {
        unsafe { Self::thread_context_raw().set(context).map_err(|_| ScopedError::Reinitialized) }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Error)]
pub enum ScopedError {
    #[error("Attempt to access an uninitialized context")]
    Uninitialized,
    #[error("Attempt to reinitialize an already initialized context")]
    Reinitialized,
}

pub type ScopedResult<T> = Result<T, ScopedError>;