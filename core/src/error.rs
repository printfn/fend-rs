use std::fmt;

pub trait Error: fmt::Display {}

pub(crate) type Never = std::convert::Infallible;

pub enum IntErr<E, I: Interrupt> {
    Interrupt(I::Int),
    Error(E),
}

#[allow(clippy::use_self)]
impl<E, I: Interrupt> IntErr<E, I> {
    pub fn expect(self, msg: &'static str) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("{}", msg),
        }
    }

    pub fn unwrap(self) -> IntErr<Never, I> {
        match self {
            Self::Interrupt(i) => IntErr::<Never, I>::Interrupt(i),
            Self::Error(_) => panic!("Unwrap"),
        }
    }

    pub fn map<F>(self, f: impl FnOnce(E) -> F) -> IntErr<F, I> {
        match self {
            Self::Interrupt(i) => IntErr::Interrupt(i),
            Self::Error(e) => IntErr::Error(f(e)),
        }
    }
}

#[allow(clippy::use_self)]
impl<E: fmt::Display, I: Interrupt> IntErr<E, I> {
    pub fn into_string(self) -> IntErr<String, I> {
        match self {
            Self::Interrupt(i) => IntErr::<String, I>::Interrupt(i),
            Self::Error(e) => IntErr::<String, I>::Error(e.to_string()),
        }
    }
}

impl<E, I: Interrupt> From<E> for IntErr<E, I> {
    fn from(e: E) -> Self {
        Self::Error(e)
    }
}

#[allow(clippy::use_self)]
impl<E: Error, I: Interrupt> From<IntErr<Never, I>> for IntErr<E, I> {
    fn from(e: IntErr<Never, I>) -> Self {
        match e {
            IntErr::Error(never) => match never {},
            IntErr::Interrupt(i) => Self::Interrupt(i),
        }
    }
}

impl<E: std::fmt::Debug, I: Interrupt> std::fmt::Debug for IntErr<E, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Interrupt(i) => write!(f, "{:?}", i)?,
            Self::Error(e) => write!(f, "{:?}", e)?,
        }
        Ok(())
    }
}

impl Error for std::fmt::Error {}
impl Error for String {}

pub trait Interrupt {
    type Int: fmt::Debug;
    fn test(&self) -> Result<(), Self::Int>;
}

#[derive(Default)]
pub(crate) struct NeverInterrupt {}
impl Interrupt for NeverInterrupt {
    type Int = std::convert::Infallible;
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
    }
}

pub(crate) struct PossibleInterrupt {}
impl Interrupt for PossibleInterrupt {
    type Int = ();
    fn test(&self) -> Result<(), Self::Int> {
        Ok(())
    }
}
