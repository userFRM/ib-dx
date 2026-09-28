//! ibapi-compatible EWrapper base class with no-op default callbacks.

use pyo3::prelude::*;

use super::class_reports::ErrorOrigin;

/// ibapi-compatible EWrapper base class.
/// Users subclass this in Python and override callbacks they care about.
/// All methods are no-ops by default.
///
/// Holds nothing and is frozen, so an instance is laid out exactly as
/// `object`'s are. The reference client's own sample program is
/// `class App(EWrapper, EClient)`, and the interpreter refuses a class whose
/// bases lay an instance out two different ways — so with a field or a borrow
/// flag here, that class cannot be defined at all. `EClient` carries the
/// state; this class must go on carrying none.
#[pyclass(frozen, subclass)]
pub struct EWrapper;

#[pymethods]
#[allow(unused_variables)] // the defaults are no-ops; the names are what a keyword call needs
impl EWrapper {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &Bound<'_, pyo3::types::PyTuple>, kwargs: Option<&Bound<'_, pyo3::types::PyDict>>) -> Self {
        Self
    }

    /// Answer to the name the reference client gives a callback as well as the
    /// name this one gives it.
    ///
    /// Every callback here is named with underscores. Code written for the
    /// reference client names them with the words run together, and asks the
    /// base class about them — whether a subclass overrode one, or by calling
    /// the default through `super()`. Under this class those names were simply
    /// absent.
    ///
    /// Only reached when the attribute was not found, so it costs nothing on
    /// the names this class defines, and a name that names no callback is still
    /// refused rather than answered with a do-nothing.
    ///
    /// Carries the client's aliases as well: in `App(EWrapper, EClient)` the
    /// interpreter takes this hook for the whole instance and never asks the
    /// client's, so `app.eConnect` has to be answered from here. On a wrapper
    /// alone they name nothing and are refused as before.
    fn __getattr__(slf: Bound<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
        super::contract::by_reference_name(slf.as_any(), name, super::client::REFERENCE_ALIASES)
    }

    // ── Connection ──

    /// The session is open. Nothing has been asked for yet.
    fn connect_ack(&self) {}

    /// The session is over, because this client ended it. A session that
    /// went away instead is reported on `error` under 1100.
    fn connection_closed(&self) {}

    /// The first order id this session may use. Each order needs one
    /// higher than the last.
    fn next_valid_id(&self, order_id: i64) {}

    /// Every account this login may act for, separated by commas. One
    /// for most logins; an advisor has several.
    fn managed_accounts(&self, accounts_list: &str) {}

    #[pyo3(signature = (req_id, error_time, error_code, error_string, advanced_order_reject_json=""))]
    /// What the venue said about a request, under the number it says it
    /// with. Codes from 2100 to 2200 are notices about a connection rather than
    /// failures. `req_id` is -1 for anything that answers no particular request.
    ///
    /// A request this client will not send is reported here too, under the same
    /// numbers the reference client uses: 321 for a request that fails
    /// validation, 200 for a contract description that matches nothing, 504
    /// for a call made with no session.
    ///
    /// `error_time` is the reference client's second parameter, in
    /// milliseconds since the epoch. A gateway stamps every error it sends,
    /// and so does this client: what the venue said about an order on a
    /// report, a refusal or a message, with the second the report states it
    /// was sent, and every other error with the clock as it is delivered.
    fn error(
        &self,
        req_id: i64,
        error_time: i64,
        error_code: i64,
        error_string: &str,
        advanced_order_reject_json: &str,
    ) {
    }

    #[pyo3(signature = (origin, error_time, error_code, error_string, advanced_order_reject_json=""))]
    /// An error, with what it is about: a request and whether nothing more
    /// follows for it, an order and the operation it answers, a request that
    /// carries no number, the session, or a lookup this client made for
    /// itself. The number `error` carries can be any of these and does not
    /// say which; `origin` says.
    ///
    /// Every error reaches a subclass of this class here. By default it goes on
    /// to `error`, under the number `origin.id` states it under, so a subclass
    /// that overrides only `error` is told exactly what it was told before. A
    /// wrapper that is not a subclass and has no method of this name is called
    /// on `error`, as the reference client calls it.
    fn error_from(
        slf: &Bound<'_, Self>,
        origin: &Bound<'_, ErrorOrigin>,
        error_time: i64,
        error_code: i64,
        error_string: &str,
        advanced_order_reject_json: &str,
    ) -> PyResult<()> {
        let id = origin.get().0.id();
        slf.call_method1("error", (id, error_time, error_code, error_string, advanced_order_reject_json))?;
        Ok(())
    }

    /// The venue clock, in seconds since the epoch.
    fn current_time(&self, time: i64) {}

    /// The venue clock, in milliseconds since the epoch.
    ///
    /// The same clock `current_time` reports, at the precision the venue
    /// stated it in. The stamp can carry a fraction of a second and this reads
    /// it where it does, but the stamps measured against this venue carried
    /// none, so the answer lands on a whole second.
    fn current_time_in_millis(&self, time_in_millis: i64) {}

    // ── Market Data ──

    /// One price of a quote, and which price it is. `tick_type` names it —
    /// 1 bid, 2 ask, 4 last, 9 close — and `attrib` says whether it can be traded
    /// against and whether it is past its limit. A size arrives on `tick_size`
    /// under the type that belongs to it.
    fn tick_price(&self, req_id: i64, tick_type: i32, price: f64, attrib: Py<PyAny>) {}

    /// One size of a quote, and which size it is: 0 bid, 3 ask, 5 last, 8
    /// the day's volume.
    fn tick_size(&self, req_id: i64, tick_type: i32, size: Py<PyAny>) {}

    /// A quote's value that is not a number — a timestamp, an exchange
    /// map, a set of ids.
    fn tick_string(&self, req_id: i64, tick_type: i32, value: &str) {}

    /// A quote's value that is a number and is not a price or a size —
    /// an implied volatility, an index future's premium, a halt.
    fn tick_generic(&self, req_id: i64, tick_type: i32, value: f64) {}

    /// A snapshot has stated everything it is going to. Only for a
    /// subscription asked for as a snapshot; a streaming one never ends.
    fn tick_snapshot_end(&self, req_id: i64) {}

    /// Which feed a subscription is being served from: 1 live, 2 frozen,
    /// 3 delayed, 4 delayed and frozen.
    fn market_data_type(&self, req_id: i64, market_data_type: i32) {}

    // ── Orders ──

    /// Where an order stands now: stated as the venue reports on the order,
    /// again on each fill, and after the order's `open_order` when open orders
    /// are asked for. Sending a cancel states nothing, as a gateway states
    /// nothing then: asked for, the order reads `PendingCancel`, and the
    /// venue's next report on it states it so. `filled` and `remaining` are
    /// shares, `avg_fill_price` the average of what has filled so far.
    fn order_status(
        &self, order_id: i64, status: &str, filled: Py<PyAny>, remaining: Py<PyAny>,
        avg_fill_price: f64, perm_id: i64, parent_id: i64,
        last_fill_price: f64, client_id: i64, why_held: &str, mkt_cap_price: f64,
    ) {}

    /// An order as the venue holds it, and the state it is in. Fires
    /// beside every `order_status`, when open orders are asked for, and once for
    /// a preview — where the state carries what the order would cost and no
    /// status follows, because a preview is not an order.
    fn open_order(&self, order_id: i64, contract: Py<PyAny>, order: Py<PyAny>, order_state: Py<PyAny>) {}

    /// Every open order has been stated.
    fn open_order_end(&self) {}

    /// One fill, against the order and contract it filled. What it cost
    /// arrives separately, on `commission_and_fees_report`.
    fn exec_details(&self, req_id: i64, contract: Py<PyAny>, execution: Py<PyAny>) {}

    /// Every execution answering this request has been stated.
    fn exec_details_end(&self, req_id: i64) {}

    /// What a fill cost, matched to it by execution id.
    fn commission_and_fees_report(&self, commission_and_fees_report: Py<PyAny>) {}

    // ── Account ──

    /// One figure the venue states about an account, in the currency it
    /// states it in. An account is stated in several currencies at once, so the
    /// same key arrives more than once.
    fn update_account_value(&self, key: &str, value: &str, currency: &str, account_name: &str) {}

    /// One position, as the venue values it now.
    fn update_portfolio(
        &self, contract: Py<PyAny>, position: Py<PyAny>, market_price: f64,
        market_value: f64, average_cost: f64, unrealized_pnl: f64,
        realized_pnl: f64, account_name: &str,
    ) {}

    /// When the account figures above were last stated.
    fn update_account_time(&self, timestamp: &str) {}

    /// The account has been fully stated. Fires once the venue has
    /// stopped adding to it, not on the first figure.
    fn account_download_end(&self, account: &str) {}

    /// One figure answering `req_account_summary`, in the currency the
    /// venue states it in.
    fn account_summary(&self, req_id: i64, account: &str, tag: &str, value: &str, currency: &str) {}

    /// Every figure answering this request has been stated.
    fn account_summary_end(&self, req_id: i64) {}

    /// One position held, on any account this login may act for.
    fn position(&self, account: &str, contract: Py<PyAny>, pos: Py<PyAny>, avg_cost: f64) {}

    /// Every position has been stated.
    fn position_end(&self) {}

    /// An account's running profit: today's, what is unrealised, and what
    /// has been realised.
    fn pnl(&self, req_id: i64, daily_pnl: f64, unrealized_pnl: f64, realized_pnl: f64) {}

    /// The same for one position, with the size held.
    fn pnl_single(
        &self, req_id: i64, pos: Py<PyAny>, daily_pnl: f64,
        unrealized_pnl: f64, realized_pnl: f64, value: f64,
    ) {}

    // ── Historical Data ──

    /// One bar answering a historical request. `bar.date` is a day for a
    /// daily bar and a moment for anything shorter, in the zone the bar carries.
    fn historical_data(&self, req_id: i64, bar: Py<PyAny>) {}

    /// Every bar answering this request has been stated, and the window
    /// they cover.
    fn historical_data_end(&self, req_id: i64, start: &str, end: &str) {}

    /// A bar that continues a `keep_up_to_date` request, after its
    /// first batch completed. The bar still forming is restated as it changes.
    fn historical_data_update(&self, req_id: i64, bar: Py<PyAny>) {}

    /// The earliest moment the venue holds data for a contract.
    fn head_timestamp(&self, req_id: i64, head_timestamp: &str) {}

    // ── Contract Details ──

    /// One contract matching a description, with everything the venue
    /// states about it. A description can match more than one.
    fn contract_details(&self, req_id: i64, contract_details: Py<PyAny>) {}

    /// Every contract matching this request has been stated.
    fn contract_details_end(&self, req_id: i64) {}

    /// Contracts whose symbol or name matches a pattern, across venues.
    fn symbol_samples(&self, req_id: i64, contract_descriptions: Py<PyAny>) {}

    // ── Tick-by-Tick ──

    /// One trade, as it happens. `tick_attrib_last` says whether it was
    /// past a limit and whether it goes unreported to the tape.
    fn tick_by_tick_all_last(
        &self, req_id: i64, tick_type: i32, time: i64, price: f64,
        size: Py<PyAny>, tick_attrib_last: Py<PyAny>, exchange: &str, special_conditions: &str,
    ) {}

    /// One change to the top of the book, as it happens.
    fn tick_by_tick_bid_ask(
        &self, req_id: i64, time: i64, bid_price: f64, ask_price: f64,
        bid_size: Py<PyAny>, ask_size: Py<PyAny>, tick_attrib_bid_ask: Py<PyAny>,
    ) {}

    /// One change to the midpoint, as it happens.
    fn tick_by_tick_mid_point(&self, req_id: i64, time: i64, mid_point: f64) {}

    // ── Scanner ──

    /// One row of a scan, in rank order.
    fn scanner_data(
        &self, req_id: i64, rank: i32, contract_details: Py<PyAny>,
        distance: &str, benchmark: &str, projection: &str, legs_str: &str,
    ) {}

    /// Every row of this scan has been stated.
    fn scanner_data_end(&self, req_id: i64) {}

    /// Every scan the venue offers and what each can be filtered by, as
    /// the XML the venue publishes.
    fn scanner_parameters(&self, xml: &str) {}

    // ── News ──

    /// Every news provider this account may read.
    fn news_providers(&self, news_providers: Py<PyAny>) {}

    /// The body of one article. `article_type` is 0 for text and 1 for a
    /// binary document.
    fn news_article(&self, req_id: i64, article_type: i32, article_text: &str) {}

    /// One headline from the archive.
    fn historical_news(
        &self, req_id: i64, time: &str, provider_code: &str,
        article_id: &str, headline: &str,
    ) {}

    /// Every headline answering this request has been stated, and
    /// whether the archive holds more.
    fn historical_news_end(&self, req_id: i64, has_more: bool) {}

    /// A headline about a contract being watched, as it is published.
    fn tick_news(
        &self, ticker_id: i64, time_stamp: i64, provider_code: &str,
        article_id: &str, headline: &str, extra_data: &str,
    ) {}

    // ── Market Depth ──

    /// One level of a book that names no venue. `operation` is 0 to
    /// insert, 1 to update, 2 to delete; `side` is 0 ask, 1 bid.
    fn update_mkt_depth(
        &self, req_id: i64, position: i32, operation: i32,
        side: i32, price: f64, size: Py<PyAny>,
    ) {}

    /// One level of a book that names the venue it stands on. Every
    /// level from this client names one.
    fn update_mkt_depth_l2(
        &self, req_id: i64, position: i32, market_maker: &str,
        operation: i32, side: i32, price: f64, size: Py<PyAny>, is_smart_depth: bool,
    ) {}

    // ── Market Depth (additional) ──

    /// Every exchange the venue names, in the two sections it names
    /// them in: shares and derivatives.
    fn mkt_depth_exchanges(&self, depth_mkt_data_descriptions: Py<PyAny>) {}

    // ── Real-Time Bars ──

    /// One five-second bar of a live stream.
    fn real_time_bar(
        &self, req_id: i64, date: i64, open: f64, high: f64,
        low: f64, close: f64, volume: Py<PyAny>, wap: Py<PyAny>, count: i32,
    ) {}

    // ── Historical Ticks ──

    /// Historical midpoints, in batches, until `done`.
    fn historical_ticks(&self, req_id: i64, ticks: Py<PyAny>, done: bool) {}

    /// Historical quotes, in batches, until `done`.
    fn historical_ticks_bid_ask(&self, req_id: i64, ticks: Py<PyAny>, done: bool) {}

    /// Historical trades, in batches, until `done`.
    fn historical_ticks_last(&self, req_id: i64, ticks: Py<PyAny>, done: bool) {}

    // ── Options ──

    /// The venue's model for an option: the volatility its price implies, the
    /// greeks, and the modelled value of the option and its underlying.
    ///
    /// Every figure is `None` where the venue stated none. The reference
    /// client hands a caller `None` for those, so this surface does too, and a
    /// wrapper that has not overridden this is handed the same. Declared as a
    /// number, the default refused the call and the exception left the caller's
    /// reading loop: a caller who watched an option and did not write this
    /// method lost the session on the first model the venue did not fill in.
    fn tick_option_computation(
        &self, req_id: i64, tick_type: i32, tick_attrib: i32,
        implied_vol: Option<f64>, delta: Option<f64>, opt_price: Option<f64>,
        pv_dividend: Option<f64>, gamma: Option<f64>, vega: Option<f64>,
        theta: Option<f64>, und_price: Option<f64>,
    ) {}

    /// One venue's option chain for an underlying and trading class: the
    /// expiries and strikes it lists for that class.
    fn security_definition_option_parameter(
        &self, req_id: i64, exchange: &str, underlying_con_id: i64,
        trading_class: &str, multiplier: &str, expirations: Py<PyAny>, strikes: Py<PyAny>,
    ) {}

    /// Every venue's chain has been stated.
    fn security_definition_option_parameter_end(&self, req_id: i64) {}

    // ── Fundamental Data ──

    /// A fundamental report, as the XML the venue publishes.
    fn fundamental_data(&self, req_id: i64, data: &str) {}

    // ── News Bulletins ──

    /// A notice the venue broadcasts to everyone — an exchange
    /// unavailable, a system message.
    fn update_news_bulletin(&self, msg_id: i64, msg_type: i32, message: &str, orig_exchange: &str) {}

    // ── Financial Advisor ──

    /// A partition of an advisor's configuration, as the XML the venue
    /// holds it in.
    fn receive_fa(&self, fa_data_type: i32, xml: &str) {}

    /// An advisor configuration has been replaced.
    fn replace_fa_end(&self, req_id: i64, text: &str) {}

    // ── Multi-Account / Multi-Model ──

    /// One position, for a request naming an account or a model.
    fn position_multi(&self, req_id: i64, account: &str, model_code: &str, contract: Py<PyAny>, pos: Py<PyAny>, avg_cost: f64) {}

    /// Every position answering this request has been stated.
    fn position_multi_end(&self, req_id: i64) {}

    /// One account figure, for a request naming an account or a model.
    fn account_update_multi(&self, req_id: i64, account: &str, model_code: &str, key: &str, value: &str, currency: &str) {}

    /// Every figure answering this request has been stated.
    fn account_update_multi_end(&self, req_id: i64) {}

    // ── Tier 3: Display Groups ──

    /// Which display groups exist, as the venue numbers them.
    fn display_group_list(&self, req_id: i64, groups: &str) {}

    /// What a display group is now showing.
    fn display_group_updated(&self, req_id: i64, contract_info: &str) {}

    // ── Tier 3: Market Rules ──

    /// The price ladder a contract trades on: each step, and what the
    /// price moves in above it.
    fn market_rule(&self, market_rule_id: i64, price_increments: Py<PyAny>) {}

    // ── Tier 3: Smart Components ──

    /// Which venue each bit of a quote's exchange mask refers to, and
    /// the letter that venue is named by.
    fn smart_components(&self, req_id: i64, smart_component_map: Py<PyAny>) {}

    // ── Tier 3: Soft Dollar Tiers ──

    /// The soft dollar tiers this account may direct commission to.
    fn soft_dollar_tiers(&self, req_id: i64, tiers: Py<PyAny>) {}

    // ── Tier 3: Family Codes ──

    /// The account families this login belongs to.
    fn family_codes(&self, family_codes: Py<PyAny>) {}

    // ── Tier 3: Histogram Data ──

    /// How much traded at each price over a window.
    fn histogram_data(&self, req_id: i64, items: Py<PyAny>) {}

    // ── Tier 3: User Info ──

    /// What the login is entitled to, as the venue states it.
    fn user_info(&self, req_id: i64, white_branding_id: &str) {}

    // ── Tier 3: WSH ──

    /// What the corporate-events calendar carries: its event types and
    /// the fields each one has, as the JSON the venue publishes.
    fn wsh_meta_data(&self, req_id: i64, data_json: &str) {}

    /// Events from the corporate-events calendar, as the JSON the venue
    /// publishes. Events themselves need a Wall Street Horizon subscription; a
    /// login without one is answered with an empty set.
    fn wsh_event_data(&self, req_id: i64, data_json: &str) {}

    // ── Tier 3: Completed Orders ──

    /// An order that is done — filled, cancelled or expired — as the
    /// venue holds it.
    fn completed_order(&self, contract: Py<PyAny>, order: Py<PyAny>, order_state: Py<PyAny>) {}

    /// Every completed order has been stated.
    fn completed_orders_end(&self) {}

    // ── Tier 3: Order Bound ──

    /// An order placed elsewhere has been bound to this session, so its
    /// changes arrive here.
    fn order_bound(&self, order_id: i64, api_client_id: i64, api_order_id: i64) {}

    // ── Tier 3: Tick Req Params ──

    /// What a subscription was given: the increment its prices move in,
    /// which venues it is served from, and which feed answered. Sent once per
    /// request, with the first record it is served, as a gateway sends it; a
    /// request served none is sent none.
    fn tick_req_params(&self, ticker_id: i64, min_tick: f64, bbo_exchange: &str, snapshot_permissions: i64) {}

    // ── Tier 3: Bond Contract Details ──

    /// One bond matching a description, with its terms: what it pays,
    /// how and when, whether it can be called or put, whether it converts, what
    /// it is rated.
    fn bond_contract_details(&self, req_id: i64, contract_details: Py<PyAny>) {}

    // ── Tier 3: declared by the TWS API, and not fired here ──
    //
    // Each exists so a program written against that API runs against this
    // one, and each says why it stays silent.

    /// The contract a market-data request should be asked for under instead.
    ///
    /// A gateway sends it, and asks the venue nothing, for a contract for
    /// difference whose own definition asks for its underlying's data, where
    /// the account's logon permissions allow that: the underlying's contract
    /// and the venue to ask on. This client does not read a definition's flags
    /// for asking on the underlying before subscribing: the request goes to
    /// the venue as asked, and this never fires.
    fn reroute_mkt_data_req(&self, req_id: i64, con_id: i64, exchange: &str) {}

    /// The same, for a request for the book rather than the quote.
    fn reroute_mkt_depth_req(&self, req_id: i64, con_id: i64, exchange: &str) {}

    /// The contract the venue paired with a delta-neutral order.
    ///
    /// Declared by the TWS API and never fired on a gateway: a gateway never
    /// sends it. It fires here as it does there: never.
    fn delta_neutral_validation(&self, req_id: i64, delta_neutral_contract: Py<PyAny>) {}

    /// An exchange-for-physical quote. Declared by the TWS API and never fired
    /// on a gateway, so it fires here as it does there: never.
    #[allow(clippy::too_many_arguments)]
    fn tick_efp(
        &self, req_id: i64, tick_type: i32, basis_points: f64,
        formatted_basis_points: &str, implied_future: f64, hold_days: i32,
        future_last_trade_date: &str, dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    ) {}

    /// A step in the TWS API's verification handshake. Declared by the TWS API
    /// and never fired on a gateway, so these four fire here as they do there:
    /// never.
    fn verify_message_api(&self, api_data: &str) {}
    /// Whether that handshake was accepted.
    fn verify_completed(&self, is_successful: bool, error_text: &str) {}
    /// The same handshake, where the terminal also authenticates the program.
    fn verify_and_auth_message_api(&self, api_data: &str, xyz_challenge: &str) {}
    /// Whether that one was accepted.
    fn verify_and_auth_completed(&self, is_successful: bool, error_text: &str) {}

    /// Declared by the TWS API as `winError`. No message on the wire carries
    /// it, and the TWS API's Python client declares it and never raises it, so
    /// it fires here as it does there: never. Here, trouble on a connection
    /// reaches a caller on the error callback.
    fn win_error(&self, text: &str, last_error: i32) {}

    // ── Tier 3: Historical Schedule ──

    /// When a contract's venue was open over a window, session by
    /// session, in the zone the venue keeps.
    fn historical_schedule(&self, req_id: i64, start_date_time: &str, end_date_time: &str, time_zone: &str, sessions: Py<PyAny>) {}
}

/// Register EWrapper on the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EWrapper>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The constructor takes the arguments PyO3 passes a `#[new]`, so the only
    /// way to build one is from Python. That the type is constructible at all
    /// is what the compiler already checks; this asserts that the type reaches
    /// Python carrying callbacks on it.
    ///
    /// Which callbacks, and all of them, is asserted where the list can be
    /// written out without repeating it in two languages:
    /// `tests/python/test_the_callback_surface_is_complete.py`. Five names
    /// here would pass while the other seventy-six were missing.
    #[test]
    fn ewrapper_reaches_python_carrying_callbacks() {
        Python::initialize();
        Python::attach(|py| {
            let cls = py.get_type::<EWrapper>();
            for method in ["error", "tick_price", "tick_size", "next_valid_id", "connection_closed"] {
                assert!(
                    cls.hasattr(method).unwrap(),
                    "EWrapper must expose {method}() for a subclass to override",
                );
            }
        });
    }
}
