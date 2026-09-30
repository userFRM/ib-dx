//! What an order waits for before it works.
//!
//! Every field a condition carries is born unset, as the reference client
//! births them, and an order placed holding one that was never set is refused
//! under its id naming the field: the reference refuses to send a value nobody
//! stated, and a default of zero or true here went on the wire as a trigger
//! the caller never asked for — the opposite direction from the one an omitted
//! reading meant.
//!
//! How a condition reads as text — its sentence and its repr — is written on
//! the Python surface, in the reference client's own words; nothing here
//! renders one.

// The other families, and the helper every class here uses.
use super::contract::set_from_keywords;
use pyo3::prelude::*;

use super::{camel_aliases_copy, camel_aliases_owned};
use crate::types::*;
use super::super::types::PRICE_SCALE_F;

/// Price condition: trigger when an instrument's price crosses a threshold.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PriceCondition {
    #[pyo3(get, set)]
    pub con_id: Option<i64>,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
    #[pyo3(get, set)]
    pub price: Option<f64>,
    #[pyo3(get, set)]
    pub is_more: Option<bool>,
    #[pyo3(get, set)]
    pub trigger_method: Option<i32>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl PriceCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(conId)]
    fn get_con_id_alias(&self) -> Option<i64> { self.con_id }
    #[setter(conId)]
    fn set_con_id_alias(&mut self, v: Option<i64>) { self.con_id = v; }

    #[new]
    // Unset, which is what the reference client holds for every field nobody
    // named: a value invented here goes on the wire as a trigger the caller
    // never stated.
    #[pyo3(signature = (trigger_method=None, con_id=None, exchange=None, is_more=None, price=None, is_conjunction_connection=true, **keywords))]
    fn new(trigger_method: Option<i32>, con_id: Option<i64>, exchange: Option<String>, is_more: Option<bool>, price: Option<f64>, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { con_id, exchange, price, is_more, trigger_method, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl PriceCondition {
    /// A condition this client cannot carry as stated is refused rather than
    /// changed: a field nobody set, sent as an invented default, waits on
    /// something the caller never stated, and a price the wire's fixed point
    /// cannot hold, converted in silence, waits on a number nobody stated.
    ///
    /// The trigger method is an `int`, as the TWS API carries it, and goes to
    /// the venue as stated: no gateway refusal of any value has been read.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let is_more = self.is_more.ok_or("a price condition states no isMore")?;
        let price = self.price.ok_or("a price condition states no price")?;
        let con_id = self.con_id.ok_or("a price condition states no conId")?;
        let exchange = self.exchange.clone().ok_or("a price condition states no exchange")?;
        let trigger_method = self.trigger_method.ok_or("a price condition states no triggerMethod")?;
        crate::client_core::require_finite_price("a price condition's price", price)?;
        Ok(OrderCondition::Price {
            con_id,
            exchange,
            price: crate::types::price_from_f64(price),
            is_more,
            trigger_method,
            is_conjunction_connection: self.is_conjunction_connection,
        })
    }
}

/// Time condition: trigger at a specific time.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct TimeCondition {
    #[pyo3(get, set)]
    pub time: Option<String>,
    #[pyo3(get, set)]
    pub is_more: Option<bool>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl TimeCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(isMore)]
    fn get_is_more_alias(&self) -> Option<bool> { self.is_more }
    #[setter(isMore)]
    fn set_is_more_alias(&mut self, v: Option<bool>) { self.is_more = v; }

    #[new]
    #[pyo3(signature = (is_more=None, time=None, is_conjunction_connection=true, **keywords))]
    fn new(is_more: Option<bool>, time: Option<String>, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { time, is_more, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl TimeCondition {
    /// A field nobody set is refused rather than sent as an invented value.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let is_more = self.is_more.ok_or("a time condition states no isMore")?;
        let time = self.time.clone().ok_or("a time condition states no time")?;
        Ok(OrderCondition::Time { time, is_more, is_conjunction_connection: self.is_conjunction_connection })
    }
}

/// Margin condition: trigger based on margin cushion percentage.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct MarginCondition {
    #[pyo3(get, set)]
    pub percent: Option<i32>,
    #[pyo3(get, set)]
    pub is_more: Option<bool>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl MarginCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(isMore)]
    fn get_is_more_alias(&self) -> Option<bool> { self.is_more }
    #[setter(isMore)]
    fn set_is_more_alias(&mut self, v: Option<bool>) { self.is_more = v; }

    #[new]
    #[pyo3(signature = (is_more=None, percent=None, is_conjunction_connection=true, **keywords))]
    fn new(is_more: Option<bool>, percent: Option<i32>, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { percent, is_more, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl MarginCondition {
    /// A field nobody set is refused rather than sent as an invented value.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let is_more = self.is_more.ok_or("a margin condition states no isMore")?;
        let percent = self.percent.ok_or("a margin condition states no percent")?;
        Ok(OrderCondition::Margin { percent, is_more, is_conjunction_connection: self.is_conjunction_connection })
    }
}

/// Execution condition: trigger on trade execution.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct ExecutionCondition {
    #[pyo3(get, set)]
    pub symbol: Option<String>,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
    #[pyo3(get, set)]
    pub sec_type: Option<String>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl ExecutionCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(exch)]
    fn get_exchange_alias(&self) -> Option<String> { self.exchange.clone() }
    #[setter(exch)]
    fn set_exchange_alias(&mut self, v: Option<String>) { self.exchange = v; }

    #[new]
    #[pyo3(signature = (sec_type=None, exchange=None, symbol=None, is_conjunction_connection=true, **keywords))]
    fn new(sec_type: Option<String>, exchange: Option<String>, symbol: Option<String>, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { symbol, exchange, sec_type, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl ExecutionCondition {
    /// A field nobody set is refused rather than sent as an invented value.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let sec_type = self.sec_type.clone().ok_or("an execution condition states no secType")?;
        let exchange = self.exchange.clone().ok_or("an execution condition states no exchange")?;
        let symbol = self.symbol.clone().ok_or("an execution condition states no symbol")?;
        Ok(OrderCondition::Execution {
            symbol,
            exchange,
            sec_type,
            is_conjunction_connection: self.is_conjunction_connection,
        })
    }
}

/// Volume condition: trigger when volume exceeds a threshold.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct VolumeCondition {
    #[pyo3(get, set)]
    pub con_id: Option<i64>,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
    #[pyo3(get, set)]
    pub volume: Option<i64>,
    #[pyo3(get, set)]
    pub is_more: Option<bool>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl VolumeCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(conId)]
    fn get_con_id_alias(&self) -> Option<i64> { self.con_id }
    #[setter(conId)]
    fn set_con_id_alias(&mut self, v: Option<i64>) { self.con_id = v; }

    #[new]
    #[pyo3(signature = (con_id=None, exchange=None, is_more=None, volume=None, is_conjunction_connection=true, **keywords))]
    fn new(con_id: Option<i64>, exchange: Option<String>, is_more: Option<bool>, volume: Option<i64>, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { con_id, exchange, volume, is_more, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl VolumeCondition {
    /// A field nobody set is refused rather than sent as an invented value.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let is_more = self.is_more.ok_or("a volume condition states no isMore")?;
        let volume = self.volume.ok_or("a volume condition states no volume")?;
        let con_id = self.con_id.ok_or("a volume condition states no conId")?;
        let exchange = self.exchange.clone().ok_or("a volume condition states no exchange")?;
        Ok(OrderCondition::Volume {
            con_id,
            exchange,
            volume,
            is_more,
            is_conjunction_connection: self.is_conjunction_connection,
        })
    }
}

/// Percentage change condition: trigger on % change from close.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PercentChangeCondition {
    #[pyo3(get, set)]
    pub con_id: Option<i64>,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
    /// Born at the reference client's own unset marker for a double rather
    /// than `None`, as it is born there: `UNSET_DOUBLE`, which is `f64::MAX`.
    #[pyo3(get, set)]
    pub change_percent: f64,
    #[pyo3(get, set)]
    pub is_more: Option<bool>,
    #[pyo3(get, set)]
    pub is_conjunction_connection: bool,
}

#[pymethods]
impl PercentChangeCondition {
    // The reference client spells these by running the words together, and
    // names an exchange `exch`. A condition built the way that client builds
    // it sets them by those names.
    #[getter(conId)]
    fn get_con_id_alias(&self) -> Option<i64> { self.con_id }
    #[setter(conId)]
    fn set_con_id_alias(&mut self, v: Option<i64>) { self.con_id = v; }

    #[new]
    #[pyo3(signature = (con_id=None, exchange=None, is_more=None, change_percent=f64::MAX, is_conjunction_connection=true, **keywords))]
    fn new(con_id: Option<i64>, exchange: Option<String>, is_more: Option<bool>, change_percent: f64, is_conjunction_connection: bool, keywords: Option<&Bound<'_, pyo3::types::PyDict>>, py: Python<'_>) -> PyResult<Py<Self>> {
        let made = Py::new(py, Self { con_id, exchange, change_percent, is_more, is_conjunction_connection })?;
        set_from_keywords(made.bind(py).as_any(), keywords)?;
        Ok(made)
    }
}

impl PercentChangeCondition {
    /// A field nobody set is refused rather than sent as an invented value.
    /// The percent is not one of them: the reference client births it at its
    /// unset marker for a double and sends that marker as stated, so it goes
    /// as stated here too.
    pub fn to_internal(&self) -> Result<OrderCondition, String> {
        let is_more = self.is_more.ok_or("a percent-change condition states no isMore")?;
        let con_id = self.con_id.ok_or("a percent-change condition states no conId")?;
        let exchange = self.exchange.clone().ok_or("a percent-change condition states no exchange")?;
        Ok(OrderCondition::PercentChange {
            con_id,
            exchange,
            percent: self.change_percent,
            is_more,
            is_conjunction_connection: self.is_conjunction_connection,
        })
    }
}

/// A condition the engine holds, as the class a caller builds one with.
///
/// The engine keeps what a condition means rather than the object it came
/// from, so an order read back carries none and, placed again, works at once
/// with nothing holding it. Each kind the venue carries has a class here with
/// the same fields the engine keeps, so what the venue
/// reported is what comes back. What the venue reported was stated, so every
/// field comes back set.
pub(crate) fn condition_from_internal(
    py: Python<'_>,
    held: &OrderCondition,
) -> PyResult<Py<PyAny>> {
    Ok(match held {
        OrderCondition::Price { con_id, exchange, price, is_more, trigger_method, is_conjunction_connection } => {
            Py::new(py, PriceCondition {
                is_conjunction_connection: *is_conjunction_connection,
                con_id: Some(*con_id),
                exchange: Some(exchange.clone()),
                price: Some(*price as f64 / PRICE_SCALE_F),
                is_more: Some(*is_more),
                trigger_method: Some(*trigger_method),
            })?.into_any()
        }
        OrderCondition::Time { time, is_more, is_conjunction_connection } => Py::new(py, TimeCondition {
            is_conjunction_connection: *is_conjunction_connection,
            time: Some(time.clone()),
            is_more: Some(*is_more),
        })?.into_any(),
        OrderCondition::Margin { percent, is_more, is_conjunction_connection } => Py::new(py, MarginCondition {
            is_conjunction_connection: *is_conjunction_connection,
            percent: Some(*percent),
            is_more: Some(*is_more),
        })?.into_any(),
        OrderCondition::Execution { symbol, exchange, sec_type, is_conjunction_connection } => {
            Py::new(py, ExecutionCondition {
                is_conjunction_connection: *is_conjunction_connection,
                symbol: Some(symbol.clone()),
                exchange: Some(exchange.clone()),
                sec_type: Some(sec_type.clone()),
            })?.into_any()
        }
        OrderCondition::Volume { con_id, exchange, volume, is_more, is_conjunction_connection } => {
            Py::new(py, VolumeCondition {
                is_conjunction_connection: *is_conjunction_connection,
                con_id: Some(*con_id),
                exchange: Some(exchange.clone()),
                volume: Some(*volume),
                is_more: Some(*is_more),
            })?.into_any()
        }
        OrderCondition::PercentChange { con_id, exchange, percent, is_more, is_conjunction_connection } => {
            Py::new(py, PercentChangeCondition {
                is_conjunction_connection: *is_conjunction_connection,
                con_id: Some(*con_id),
                exchange: Some(exchange.clone()),
                change_percent: *percent,
                is_more: Some(*is_more),
            })?.into_any()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A price condition's trigger method and a margin condition's percent
    /// are the TWS API's `int`s, and each goes to the engine as stated: no
    /// gateway refusal of any value has been read. 5, 6, a method past what a
    /// byte holds and a negative percent included.
    #[test]
    fn a_condition_carries_its_trigger_method_and_percent_as_stated() {
        for tm in [0i32, 4, 5, 6, 7, 8, 256, -1] {
            let c = PriceCondition {
                con_id: Some(1), exchange: Some("SMART".into()), price: Some(100.0),
                is_more: Some(true), trigger_method: Some(tm), is_conjunction_connection: true,
            };
            match c.to_internal().unwrap_or_else(|e| panic!("trigger {tm} refused on a condition: {e}")) {
                OrderCondition::Price { trigger_method, .. } => assert_eq!(trigger_method, tm),
                other => panic!("not a price condition: {other:?}"),
            }
        }
        let margin = MarginCondition { percent: Some(-5), is_more: Some(false), is_conjunction_connection: true };
        assert_eq!(
            margin.to_internal().expect("a condition stated in full converts"),
            OrderCondition::Margin { percent: -5, is_more: false, is_conjunction_connection: true },
        );
    }

    /// A condition holding a field nobody set is refused naming the field,
    /// rather than placed with a value invented for it: an omitted reading
    /// went to the venue as a trigger in the opposite direction from the one
    /// the caller meant.
    #[test]
    fn a_condition_a_field_was_never_set_on_is_refused_naming_the_field() {
        let stated_but_one = PriceCondition {
            con_id: Some(1), exchange: Some("SMART".into()), price: Some(100.0),
            is_more: None, trigger_method: Some(0), is_conjunction_connection: true,
        };
        let why = stated_but_one.to_internal().expect_err("a reading nobody stated is refused");
        assert!(why.contains("isMore"), "the refusal names the field: {why}");

        let empty = TimeCondition { time: None, is_more: Some(true), is_conjunction_connection: true };
        let why = empty.to_internal().expect_err("a time nobody stated is refused");
        assert!(why.contains("time"), "the refusal names the field: {why}");
    }

    /// An order read back states what it is waiting for.
    ///
    /// The engine keeps what a condition means rather than the object it was
    /// read from, and for a while nothing built one back — so an order read
    /// back through open orders, completed orders or a reconnect came back
    /// holding none, and placing it again placed an order that worked at once.
    #[test]
    fn a_condition_read_back_is_the_condition_that_was_sent() {
        let sent = vec![
            OrderCondition::Price {
                con_id: 756733,
                exchange: "SMART".into(),
                price: (412.25 * PRICE_SCALE_F) as Price,
                is_more: true,
                trigger_method: 0,
                is_conjunction_connection: false,
            },
            OrderCondition::Time { time: "20260101 09:30:00".into(), is_more: false, is_conjunction_connection: true },
            OrderCondition::Volume {
                con_id: 756733,
                exchange: "SMART".into(),
                volume: 1_000_000,
                is_more: true,
                is_conjunction_connection: true,
            },
        ];
        let held = crate::types::model::Order { conditions: sent.clone(), ..Default::default() };

        // The test binary embeds the interpreter rather than being loaded by
        // one, so nothing has started it yet.
        Python::initialize();
        Python::attach(|py| {
            let back = super::super::class_orders::Order::from_api(py, &held)
                .expect("the order comes back");
            assert_eq!(back.conditions.bound(py).len(), sent.len(), "each one comes back");
            assert_eq!(
                back.convert_conditions(py).expect("and reads as itself"),
                sent,
                "what the venue reported is what a caller would place again",
            );
        });
    }

    /// A price condition stating a price the wire cannot hold is refused, not
    /// changed: converted in silence, the order went to the venue waiting on a
    /// zero price the caller never stated.
    #[test]
    fn a_price_condition_that_cannot_be_carried_is_refused_rather_than_changed() {
        Python::initialize();
        Python::attach(|py| {
            let holding = |condition: PriceCondition| {
                let order = super::super::class_orders::Order::default();
                order.conditions.bound(py).append(condition).unwrap();
                order
            };
            let why = holding(PriceCondition {
                con_id: Some(756733), exchange: Some("SMART".into()), price: Some(f64::NAN),
                is_more: Some(true), trigger_method: Some(0), is_conjunction_connection: true,
            })
            .convert_conditions(py)
            .expect_err("a price nobody can state is refused");
            assert!(why.contains("price"), "the refusal names the price: {why}");

        });
    }
}

camel_aliases_copy! {
    PriceCondition {
        get_is_more_alias set_is_more_alias isMore is_more Option<bool>;
        get_trigger_method_alias set_trigger_method_alias triggerMethod trigger_method Option<i32>;
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

camel_aliases_owned! {
    PriceCondition {
        get_exchange_alias set_exchange_alias exch exchange Option<String>;
    }
}

camel_aliases_owned! {
    ExecutionCondition {
        get_sec_type_alias set_sec_type_alias secType sec_type Option<String>;
    }
}

camel_aliases_copy! {
    VolumeCondition {
        get_is_more_alias set_is_more_alias isMore is_more Option<bool>;
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

camel_aliases_owned! {
    VolumeCondition {
        get_exchange_alias set_exchange_alias exch exchange Option<String>;
    }
}

camel_aliases_copy! {
    PercentChangeCondition {
        get_is_more_alias set_is_more_alias isMore is_more Option<bool>;
        get_change_percent_alias set_change_percent_alias changePercent change_percent f64;
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

camel_aliases_owned! {
    PercentChangeCondition {
        get_exchange_alias set_exchange_alias exch exchange Option<String>;
    }
}

camel_aliases_copy! {
    TimeCondition {
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

camel_aliases_copy! {
    MarginCondition {
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

camel_aliases_copy! {
    ExecutionCondition {
        get_is_conjunction_connection_alias set_is_conjunction_connection_alias isConjunctionConnection is_conjunction_connection bool;
    }
}

/// The kind a condition is, as the reference client states it: the number
/// under `condType` and `type()`, and the joins `And()` and `Or()`, which set
/// how the next condition attaches and hand the same condition back.
macro_rules! condition_kind {
    ($cls:ident, $kind:expr) => {
        #[pymethods]
        impl $cls {
            #[getter(condType)]
            fn cond_type_alias(&self) -> i32 { $kind }
            #[getter]
            fn cond_type(&self) -> i32 { $kind }
            #[pyo3(name = "type")]
            fn kind(&self) -> i32 { $kind }
            #[pyo3(name = "And")]
            fn and_join(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> { slf.is_conjunction_connection = true; slf }
            #[pyo3(name = "Or")]
            fn or_join(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> { slf.is_conjunction_connection = false; slf }
        }
    };
}

condition_kind!(PriceCondition, 1);
condition_kind!(TimeCondition, 3);
condition_kind!(MarginCondition, 4);
condition_kind!(ExecutionCondition, 5);
condition_kind!(VolumeCondition, 6);
condition_kind!(PercentChangeCondition, 7);
