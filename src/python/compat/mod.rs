//! ibapi-compatible layer: EWrapper, EClient, Contract, Order, tick types.

pub mod client;
pub mod contract;
/// The contract classes a caller works in.
pub mod class_contracts;
/// The order classes a caller works in.
pub mod class_orders;
/// What an order waits for before it works.
pub mod class_conditions;
/// What the venue reports back.
pub mod class_reports;
pub mod tick_types;
pub mod wrapper;

use pyo3::prelude::*;

/// Register all compat classes on the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    contract::register(m)?;
    tick_types::register(m)?;
    wrapper::register(m)?;
    client::register(m)?;
    Ok(())
}

/// The camelCase names the reference client gives these fields, where the whole
/// accessor is "hand the field over".
///
/// Two macros rather than one with a branch: the interpreter's own attribute
/// macro cannot see through a nested expansion, so the ownership has to be
/// decided at the call. `copy` is for fields that are copied and `owned` for
/// fields that are cloned, which keeps a clone off the ones that do not need
/// one instead of hiding it inside a macro where the lint cannot see it.
///
/// Aliases that build their answer — anything needing the interpreter, or a
/// field that is not handed over as it stands — stay written out.
macro_rules! camel_aliases_copy {
    ($cls:ident { $($get:ident $set:ident $py:ident $rust:ident $ty:ty;)* }) => {
        #[pymethods]
        impl $cls {
            $(
                #[getter($py)]
                fn $get(&self) -> $ty { self.$rust }
                #[setter($py)]
                fn $set(&mut self, v: $ty) { self.$rust = v; }
            )*
        }
    };
}

macro_rules! camel_aliases_owned {
    ($cls:ident { $($get:ident $set:ident $py:ident $rust:ident $ty:ty;)* }) => {
        #[pymethods]
        impl $cls {
            $(
                #[getter($py)]
                fn $get(&self) -> $ty { self.$rust.clone() }
                #[setter($py)]
                fn $set(&mut self, v: $ty) { self.$rust = v; }
            )*
        }
    };
}

/// The same two shapes for the fields of the data classes that state values:
/// accessors that carry None. The reference client's classes are plain
/// Python — a field takes None and reads back None, and the failure comes
/// only at send time, where the encoder raises on the None and the sending
/// function says it on the error callback under its own number. These keep
/// the typed value in the field and the field's Rust name in the class's
/// `nil` list while it is stated as None: the getter answers None for a
/// listed name, the setter lists it on None and unlists it on a value. The
/// send path reads the list before it encodes.
macro_rules! nil_aware_copy {
    ($cls:ident { $($get:ident $set:ident $py:ident $rust:ident $ty:ty;)* }) => {
        #[pymethods]
        impl $cls {
            $(
                #[getter($py)]
                fn $get(&self) -> Option<$ty> {
                    if self.nil.contains(&stringify!($rust)) { None } else { Some(self.$rust) }
                }
                #[setter($py)]
                fn $set(&mut self, v: Option<$ty>) {
                    match v {
                        None => {
                            let name = stringify!($rust);
                            if !self.nil.contains(&name) { self.nil.push(name); }
                        }
                        Some(v) => {
                            self.nil.retain(|n| *n != stringify!($rust));
                            self.$rust = v;
                        }
                    }
                }
            )*
        }
    };
}

macro_rules! nil_aware_owned {
    ($cls:ident { $($get:ident $set:ident $py:ident $rust:ident $ty:ty;)* }) => {
        #[pymethods]
        impl $cls {
            $(
                #[getter($py)]
                fn $get(&self) -> Option<$ty> {
                    if self.nil.contains(&stringify!($rust)) { None } else { Some(self.$rust.clone()) }
                }
                #[setter($py)]
                fn $set(&mut self, v: Option<$ty>) {
                    match v {
                        None => {
                            let name = stringify!($rust);
                            if !self.nil.contains(&name) { self.nil.push(name); }
                        }
                        Some(v) => {
                            self.nil.retain(|n| *n != stringify!($rust));
                            self.$rust = v;
                        }
                    }
                }
            )*
        }
    };
}

pub(crate) use {camel_aliases_copy, camel_aliases_owned, nil_aware_copy, nil_aware_owned};
