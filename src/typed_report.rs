use std::any::{Any, TypeId};
use std::backtrace::Backtrace;
use std::error::Error;
use std::ops::Deref;

pub type TypedResult<T, E> = Result<T, TypedReport<E>>;

pub struct TypedReport<T: Error + 'static> {
    error: T,
    backtrace: Backtrace,
}

impl<T: Error + 'static> TypedReport<T> {
    pub fn unwrap(self) -> T {
        self.error
    }

    pub fn inner(&self) -> &T {
        self.error.as_ref()
    }

    pub fn backtrace(&self) -> &Backtrace {
       self.backtrace.as_ref()
    }
}

impl<T: Error + 'static> Deref for TypedReport<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.error.as_ref().unwrap()
    }
}

impl<T, U, V> From<U> for TypedReport<T>
where
    T: Any + Error + 'static,
    U: Any + Error + 'static,
    V: Any + Error + 'static + From<T>,
{
    fn from(value: U) -> Self {
        let val = if TypeId::of::<U>() == TypeId::of::<TypedReport<V>>() {
            value.unwrap().into()
        } else {
            value
        };
        TypedReport {
            error: val,
            backtrace: Backtrace::capture(),
        }
    }
}

impl<T, U> From<TypedReport<T>> for TypedReport<U>
where
    T: Any + Error + 'static,
    U: Any + Error + 'static + From<T>,
{
    fn from(value: TypedReport<T>) -> Self {
        if TypeId::of::<T>() == TypeId::of::<U>() {
            return value
        }
        TypedReport {
            error: value.error.take().map(|x| x.into()),
            backtrace: value.backtrace.take(),
        }
    }
}
