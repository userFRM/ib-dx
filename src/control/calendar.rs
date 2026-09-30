//! The corporate-events calendar: what a company is about to do.
//!
//! Earnings dates, splits, dividends, meetings and the rest, from a vendor the
//! venue carries rather than from the venue itself. Two requests: what event
//! types exist, and the events themselves for contracts a caller names.
//!
//! Both ride the same envelope — one message type, one sub-protocol, told
//! apart by a number and carrying a JSON document — and both are answered on
//! that envelope too, with the answer under one number and a refusal under
//! another.
//!
//! Neither depends on the other. An event request that no metadata request
//! preceded is answered — measured on a session, which is why nothing here
//! stands in front of one.

// The query a request is built from sits beside the command that carries
// it. Reachable here because that is the path a program written against this
// client already names.
pub use crate::types::CalendarQuery;

/// The sub-protocol both requests and both answers travel under.
pub const CALENDAR_SUB_PROTOCOL: u32 = 155;

/// The tag saying which of the two requests this is.
pub const TAG_CALENDAR_REQUEST_KIND: u32 = 8081;

/// The tag carrying the request's JSON document.
pub const TAG_CALENDAR_JSON: u32 = 8082;

/// The tag a request states its own name under, and an answer echoes.
pub const TAG_CALENDAR_KEY: u32 = 6556;

/// Asking what event types exist.
pub const CALENDAR_META_DATA: u32 = 100;

/// Asking for the events themselves.
pub const CALENDAR_EVENT_DATA: u32 = 101;

/// The answer.
pub const CALENDAR_ANSWER: &str = "158";

/// A refusal, whose words are on tag 58.
pub const CALENDAR_REFUSAL: &str = "159";

/// What this client can be sent, as the venue states it: three bits, set.
///
/// Sent on both requests. It is the venue's encoding of what a caller can
/// be given, and a request that states less is answered with less.
const CLIENT_CAPABILITY: &str = "Bw==";

/// The vendor whose calendar this is.
const CALENDAR_SOURCE: &str = "WSHE";

/// The JSON asking what event types exist.
///
/// It carries no filters of any kind: the answer is the same for everybody.
pub fn meta_data_request() -> String {
    format!(
        r#"{{"T":{CALENDAR_META_DATA},"V":1,"P":{{"calendar_request":{{"client_capability":"{CLIENT_CAPABILITY}"}}}}}}"#
    )
}

/// The number a gateway refuses an event request under that names both
/// scopes, or neither.
pub const INVALID_EVENT_REQUEST: i32 = 10309;

/// The JSON asking for events.
///
/// Exactly one scope is a request a gateway takes: a named contract, or a
/// filter the caller wrote. Naming both, or neither, is refused under the
/// gateway's number before anything is built, and emptiness is judged on
/// the filter as written, which is how a gateway judges it.
///
/// Naming a contract becomes a watchlist of one, with the contract written
/// as text inside an array, which is how the venue reads it.
///
/// Nothing is stated where a caller stated nothing: a key with an empty value
/// is left out rather than sent empty.
pub fn event_data_request(
    query: &CalendarQuery,
) -> Result<String, crate::error_codes::Refusal> {
    use crate::error_codes::Refusal;
    // The gateway's only test on a contract id is against the unset marker,
    // the number the reference client leaves in a field nobody set: every
    // other value names a contract, zero and negatives included, and is
    // forwarded in the watchlist as text.
    let named = query.con_id.filter(|id| *id != i64::from(i32::MAX));
    let filtered = !query.filter.is_empty();
    if named.is_some() == filtered {
        return Err(Refusal::stated(
            INVALID_EVENT_REQUEST,
            "Invalid WSH event data request.",
        ));
    }
    let filter = if let Some(con_id) = named {
        format!(r#"{{"watchlist":["{con_id}"]}}"#)
    } else {
        // The caller's own filter, as written.
        query.filter.clone()
    };
    // On the contract-named path a gateway forces all three fill flags false
    // in the document, whatever the caller set; the caller's flags are
    // consumed only on the filter path.
    let (fill_watchlist, fill_portfolio, fill_competitors) = if named.is_some() {
        (false, false, false)
    } else {
        (query.fill_watchlist, query.fill_portfolio, query.fill_competitors)
    };

    let mut parts = vec![format!(r#""sources":["{CALENDAR_SOURCE}"]"#)];

    let mut dates: Vec<String> = Vec::new();
    if !query.start_date.trim().is_empty() {
        dates.push(format!(r#""start":"{}""#, query.start_date.trim()));
    }
    if !query.end_date.trim().is_empty() {
        dates.push(format!(r#""end":"{}""#, query.end_date.trim()));
    }
    // Always stated, empty where the caller bounded nothing. The window and
    // the account are present on every request whether or not either holds
    // anything; omitting them makes a different document.
    parts.push(format!(r#""date":{{{}}}"#, dates.join(",")));
    parts.push(r#""account":"""#.to_string());

    parts.push(format!(r#""filters":{filter}"#));
    parts.push(r#""api":true"#.to_string());
    parts.push(format!(r#""fill_watchlist":{fill_watchlist}"#));
    parts.push(format!(r#""fill_portfolio":{fill_portfolio}"#));
    parts.push(format!(r#""fill_competitors":{fill_competitors}"#));
    parts.push(r#""mode":"chronological""#.to_string());
    if let Some(limit) = query.total_limit {
        // Stated as text. A bare number is a different document.
        parts.push(format!(r#""total_limit":"{limit}""#));
    }
    parts.push(format!(r#""client_capability":"{CLIENT_CAPABILITY}""#));

    Ok(format!(
        r#"{{"T":{CALENDAR_EVENT_DATA},"V":1,"P":{{"calendar_request":{{{}}}}}}}"#,
        parts.join(","),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A gateway's only test on a contract id is against the unset marker,
    /// the field's own "nothing was set": zero and negative ids are named
    /// contracts, forwarded in the watchlist as text.
    #[test]
    fn any_contract_but_the_unset_marker_is_named() {
        for id in [0, -1] {
            let json = event_data_request(&CalendarQuery { con_id: Some(id), ..Default::default() })
                .expect("every id but the unset marker names a contract");
            assert!(json.contains(&format!(r#""watchlist":["{id}"]"#)), "{json}");
        }
        let unset = CalendarQuery { con_id: Some(i32::MAX as i64), ..Default::default() };
        let err = event_data_request(&unset).unwrap_err();
        assert!(format!("{err:?}").contains("10309"), "{err:?}");
    }

    /// The metadata request carries no filters. The answer is the same for
    /// everybody, so there is nothing to narrow.
    #[test]
    fn the_metadata_request_states_only_what_this_client_can_take() {
        let json = meta_data_request();
        assert!(json.contains(r#""T":100"#), "{json}");
        assert!(json.contains(r#""client_capability":"Bw==""#), "{json}");
        assert!(!json.contains("filters"), "nothing to filter: {json}");
    }

    /// Naming a contract becomes a watchlist of one, with the contract written
    /// as text inside an array. Written as a bare number it is a different
    /// document and the venue reads no contract at all.
    #[test]
    fn a_named_contract_becomes_a_watchlist_of_one() {
        let json = event_data_request(&CalendarQuery { con_id: Some(265598), ..Default::default() })
            .expect("a contract is enough to ask with");
        assert!(json.contains(r#""filters":{"watchlist":["265598"]}"#), "{json}");
        assert!(json.contains(r#""T":101"#), "{json}");
        assert!(json.contains(r#""sources":["WSHE"]"#), "{json}");
    }

    /// On the contract-named path a gateway forces all three fill flags
    /// false in the document, whatever the caller set; the caller's flags
    /// are consumed only on the filter path.
    #[test]
    fn a_named_contract_asks_with_no_fills() {
        let json = event_data_request(&CalendarQuery {
            con_id: Some(8314),
            fill_watchlist: true,
            fill_portfolio: true,
            fill_competitors: true,
            ..Default::default()
        })
        .expect("a named contract is a scope");
        for flag in ["fill_watchlist", "fill_portfolio", "fill_competitors"] {
            assert!(json.contains(&format!(r#""{flag}":false"#)), "{flag}: {json}");
        }

        let filtered = event_data_request(&CalendarQuery {
            filter: r#"{"earnings":true}"#.into(),
            fill_portfolio: true,
            ..Default::default()
        })
        .expect("a filter is a scope");
        assert!(filtered.contains(r#""fill_portfolio":true"#), "{filtered}");
    }

    /// A caller's own filter goes as written. The venue validates it, not this
    /// client, and rewriting it would change what was asked.
    #[test]
    fn a_callers_filter_goes_as_written() {
        let json = event_data_request(&CalendarQuery {
            filter: r#"{"portfolio":true,"other":false}"#.to_string(),
            ..Default::default()
        })
        .expect("a filter is enough to ask with");
        assert!(json.contains(r#""filters":{"portfolio":true,"other":false}"#), "{json}");
    }

    /// The window and the account are stated on every request, empty where
    /// the caller bounded nothing. A limit the caller did not set is left
    /// out.
    #[test]
    fn nothing_stated_is_nothing_sent() {
        let json = event_data_request(&CalendarQuery { con_id: Some(1), ..Default::default() })
            .expect("asks");
        assert!(json.contains(r#""date":{}"#), "an unbounded window is still stated: {json}");
        assert!(json.contains(r#""account":"""#), "the account is stated: {json}");
        assert!(!json.contains("total_limit"), "an unset limit was stated: {json}");

        let bounded = event_data_request(&CalendarQuery {
            con_id: Some(1),
            start_date: "20260810".into(),
            end_date: "20260910".into(),
            total_limit: Some(50),
            ..Default::default()
        })
        .expect("asks");
        assert!(bounded.contains(r#""date":{"start":"20260810","end":"20260910"}"#), "{bounded}");
        // Text, not a bare number.
        assert!(bounded.contains(r#""total_limit":"50""#), "{bounded}");
    }

    /// A request naming both scopes, or neither, is refused under the
    /// gateway's number and words, and nothing is built for the venue.
    #[test]
    fn a_scope_of_both_or_neither_is_refused_as_a_gateway_refuses_it() {
        let both = CalendarQuery {
            con_id: Some(8314), filter: "earnings".into(), ..Default::default()
        };
        let neither = CalendarQuery::default();
        // Emptiness is judged on the filter as written: whitespace is a
        // filter, so this names both scopes.
        let whitespace = CalendarQuery {
            con_id: Some(8314), filter: " ".into(), ..Default::default()
        };
        for query in [both, neither, whitespace] {
            let err = event_data_request(&query).unwrap_err();
            let said = format!("{err:?}");
            assert!(said.contains("10309"), "{said}");
            assert!(said.contains("Invalid WSH event data request."), "{said}");
        }
    }
}
