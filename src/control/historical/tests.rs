//! The tests for this module.
//!
//! One file per module, as `api/client` already does it. Each block below
//! reaches the code it tests through `super::super`, which is the module this
//! file belongs to.

use super::*;

#[test]
fn bar_data_type_strings() {
    assert_eq!(BarDataType::Trades.as_str(), "Last");
    assert_eq!(BarDataType::Midpoint.as_str(), "MidPoint");
    assert_eq!(BarDataType::BidAsk.as_str(), "BidAsk");
}

#[test]
fn bar_size_strings() {
    assert_eq!(BarSize::Min5.as_str(), "5 mins");
    assert_eq!(BarSize::Hour1.as_str(), "1 hour");
    assert_eq!(BarSize::Day1.as_str(), "1 day");
    assert_eq!(BarSize::Month3.as_str(), "3 months");
    assert_eq!(BarSize::Year1.as_str(), "1 year");
}

// ── single parse table, rejection instead of Min5/TRADES ──

/// The whole legal set, matched exactly: every name reads, and goes back
/// out under the spelling it was read by.
#[test]
fn bar_size_from_api_str_accepts_exactly_the_legal_names() {
    let all = [
        "1 secs", "5 secs", "10 secs", "15 secs", "30 secs",
        "1 min", "2 mins", "3 mins", "4 mins", "5 mins", "10 mins", "15 mins",
        "20 mins", "30 mins", "1 hour", "2 hours", "3 hours", "4 hours",
        "8 hours", "1 day", "1W", "1M", "3 months", "1 year",
    ];
    for s in all {
        let size = BarSize::from_api_str(s).unwrap_or_else(|e| panic!("'{s}' is legal: {e}"));
        assert_eq!(size.as_str(), s, "'{s}' goes out under the name it was read by");
    }
    assert_eq!(BarSize::from_api_str("3 months").unwrap(), BarSize::Month3);
    assert_eq!(BarSize::from_api_str("1 year").unwrap(), BarSize::Year1);
    // A quarter is three of a gateway's 31-day months and a year its 365
    // days, which is how it counts both.
    assert_eq!(BarSize::Month3.seconds(), 8_035_200);
    assert_eq!(BarSize::Year1.seconds(), 31_536_000);
}

/// A quarter and a year are named by their first day on the calendar, the
/// way a month is by its first day: the bars of one period that more than
/// one stretch answered for are joined on that day.
#[test]
fn a_quarter_and_a_year_are_named_by_their_first_day() {
    assert_eq!(period_of(BarSize::Month3, "20260517").as_deref(), Some("20260401"));
    assert_eq!(period_of(BarSize::Month3, "20260401").as_deref(), Some("20260401"));
    assert_eq!(period_of(BarSize::Month3, "20260630").as_deref(), Some("20260401"));
    assert_eq!(period_of(BarSize::Month3, "20260701").as_deref(), Some("20260701"));
    assert_eq!(period_of(BarSize::Year1, "20260517").as_deref(), Some("20260101"));
    assert_eq!(period_of(BarSize::Year1, "20261231").as_deref(), Some("20260101"));
}

/// A series of quarters or of years asked along more than one id counts its
/// bars the way a month's does, in the lengths a gateway states: a quarter of
/// 93 days and a year of 365, so a later stretch is asked for the bars still
/// wanted, in the unit the request stated.
#[test]
fn a_series_of_quarters_or_years_counts_its_bars_along_the_history() {
    use crate::control::adjustments::IdStretch;
    for (size, duration, left, length) in [
        (BarSize::Month3, "1 y", 3, "1 y"),
        (BarSize::Year1, "2 y", 2, "2 y"),
    ] {
        let req = HistoricalRequest {
            query_id: "q1".to_string(),
            con_id: 222,
            symbol: "NEWCO".to_string(),
            sec_type: "CS".to_string(),
            exchange: "BEST".to_string(),
            data_type: BarDataType::Trades,
            end_time: "20240628-20:00:00".to_string(),
            duration: duration.to_string(),
            bar_size: size,
            use_rth: true,
            keep_up_to_date: false,
            include_expired: false,
        };
        let history = vec![
            IdStretch { con_id: 222, start: "-1".into(), end: "-1".into(), symbol: String::new(), exchange: String::new() },
            IdStretch { con_id: 111, start: "-1".into(), end: "20240531".into(), symbol: String::new(), exchange: String::new() },
        ];
        let (stretches, wanted, _) = along(&req, &history).unwrap_or_else(|e| panic!("{duration}: {e}"));
        assert_eq!(stretches.len(), 2, "{duration} is asked along both ids");
        let wanted = wanted.unwrap_or_else(|| panic!("{duration} counts its bars"));
        assert_eq!(wanted.left, left, "{duration}: bars still wanted");
        assert_eq!(wanted.length(), length, "{duration}: what a later stretch is asked with");
    }
}

/// A miss is refused in a gateway's words, with the list it renders: every
/// legal size but the two longest, which are legal all the same. An alias or
/// a casing is a miss too — a gateway compares by strict equality, so what
/// the venue would have happened to answer was bars a gateway would never
/// have been asked for.
#[test]
fn bar_size_from_api_str_refuses_a_miss_in_the_gateways_words() {
    let refusal = BarSize::from_api_str("1 minute").unwrap_err();
    assert_eq!(
        refusal,
        "Historical data bar size setting is invalid. Legal ones are: \
         1 secs, 5 secs, 10 secs, 15 secs, 30 secs, 1 min, 2 mins, 3 mins, \
         4 mins, 5 mins, 10 mins, 15 mins, 20 mins, 30 mins, 1 hour, \
         2 hours, 3 hours, 4 hours, 8 hours, 1 day, 1W, 1M",
    );
    // The aliases this table once carried, and the casings it once folded.
    for s in [
        "1 sec", "1 week", "1w", "1 month", "1m",
        "1 Min", "4 MINS", "1 Day", "30 SECS",
        "1min", "7 mins", "",
    ] {
        assert_eq!(BarSize::from_api_str(s).unwrap_err(), refusal, "'{s}' is a miss");
    }
    // Every name the refusal renders is one it then reads.
    let named = refusal.split("Legal ones are: ").nth(1).expect("it names them");
    for size in named.split(", ") {
        assert!(
            BarSize::from_api_str(size).is_ok(),
            "the refusal names {size}, which it then refuses",
        );
    }
}

/// A week and a month are asked for under a name of their own.
///
/// Sent the API's own spelling the venue answers a six-month window with a
/// hundred and twenty-four bars for `1 week` and one a minute for `1 month`.
/// Sent `1W` and `1M` it answers twenty-six and six, and restates the step it
/// used. Everything shorter is asked for by its API name.
#[test]
fn a_week_and_a_month_go_out_under_the_name_the_venue_answers() {
    assert_eq!(BarSize::Week1.as_str(), "1W");
    assert_eq!(BarSize::Month1.as_str(), "1M");
    assert_eq!(BarSize::Day1.as_str(), "1 day");
    assert_eq!(BarSize::Min4.as_str(), "4 mins");
}


#[test]
fn bar_data_type_from_api_str() {
    assert_eq!(BarDataType::from_api_str("TRADES").unwrap(), BarDataType::Trades);
    assert_eq!(BarDataType::from_api_str("trades").unwrap(), BarDataType::Trades);
    assert_eq!(BarDataType::from_api_str("BID_ASK").unwrap(), BarDataType::BidAsk);
    // A name outside the table is refused rather than answered with trade
    // bars, in a gateway's words, spelled as the caller spelled it — and the
    // empty name is outside the table, rendering its two spaces.
    assert_eq!(
        BarDataType::from_api_str("TRADE"),
        Err("What to show value of TRADE rejected.".to_string()),
    );
    assert_eq!(
        BarDataType::from_api_str(""),
        Err("What to show value of  rejected.".to_string()),
    );
    assert!(BarDataType::from_api_str("BIDD").is_err());
}

#[test]
fn build_query_xml_structure() {
    let req = HistoricalRequest {
        query_id: "q1".to_string(),
        con_id: 265598,
        symbol: "AAPL".to_string(),
        sec_type: "CS".to_string(),
        exchange: "SMART".to_string(),
        data_type: BarDataType::Trades,
        end_time: "20260228-15:00:00".to_string(),
        duration: "1 d".to_string(),
        bar_size: BarSize::Min5,
        use_rth: true,
        keep_up_to_date: false,
        include_expired: false,
    };
    let xml = build_query_xml(&req);
    assert!(xml.contains("<id>q1</id>"));
    assert!(xml.contains("<contractID>265598</contractID>"));
    assert!(xml.contains("<exchange>BEST</exchange>")); // SMART→BEST
    assert!(xml.contains("<data>Last</data>"));
    assert!(xml.contains("<step>5 mins</step>"));
    assert!(xml.contains("<useRTH>true</useRTH>"));
    assert!(xml.contains("<timeLength>1 d</timeLength>"));
}

#[test]
fn build_fix_request() {
    let req = HistoricalRequest {
        query_id: "q1".to_string(),
        con_id: 265598,
        symbol: "AAPL".to_string(),
        sec_type: "CS".to_string(),
        exchange: "SMART".to_string(),
        data_type: BarDataType::Trades,
        end_time: "20260228-15:00:00".to_string(),
        duration: "1 d".to_string(),
        bar_size: BarSize::Min5,
        use_rth: true,
        keep_up_to_date: false,
        include_expired: false,
    };
    let msg = build_historical_request(&req, 1);
    let tags = fix::fix_parse(&msg);
    assert_eq!(tags[&fix::TAG_MSG_TYPE], "W");
    assert!(tags[&TAG_HISTORICAL_XML].contains("<ListOfQueries>"));
}

#[test]
fn cancel_request_structure() {
    let msg = super::build_cancel_request("12345", 1);
    let tags = fix::fix_parse(&msg);
    assert_eq!(tags[&fix::TAG_MSG_TYPE], "Z");
    assert!(tags[&TAG_HISTORICAL_XML].contains("ticker:12345"));
}

/// A reply cut mid-row is refused, not delivered as the rows it managed.
///
/// The completeness a caller reads comes off the reply's own end-of-query
/// flag, and that flag is at the front. Broken out of on the unclosed row, a
/// reply cut in the middle lost that row and everything after it — and the
/// short series that came back said it was the whole answer. The histogram
/// rows beside these are already read the other way, and say so.
#[test]
fn a_series_cut_mid_row_is_not_read_as_the_rows_it_managed() {
    let truncated = "<ResultSetBar><id>q1</id><eoq>true</eoq><tz>US/Eastern</tz>\
        <Events>\
        <Bar><time>20260227-14:30:00</time><open>272.77</open><close>269.47</close>\
        <high>272.81</high><low>269.2</low><volume>1411775</volume><count>5165</count></Bar>\
        <Bar><time>20260227-14:35:00</time><open>269.48</open><close";
    assert!(
        parse_bar_response(truncated).is_none(),
        "a reply cut short is not an answer, whatever its end-of-query flag says",
    );

    // And the whole reply still reads.
    let whole = "<ResultSetBar><id>q1</id><eoq>true</eoq><tz>US/Eastern</tz>\
        <Events>\
        <Bar><time>20260227-14:30:00</time><open>272.77</open><close>269.47</close>\
        <high>272.81</high><low>269.2</low><volume>1411775</volume><count>5165</count></Bar>\
        </Events></ResultSetBar>";
    let resp = parse_bar_response(whole).expect("a complete reply is an answer");
    assert_eq!(resp.bars.len(), 1);
}

/// The same for a trading schedule, where an unclosed session did worse than
/// vanish: the session before it took the next one's closing time.
#[test]
fn a_schedule_cut_mid_session_is_not_read_as_the_sessions_it_managed() {
    let truncated = "<ResultSetSchedule><id>q1</id><tz>US/Eastern</tz>\
        <Open><refDate>20260227</refDate><time>20260227-09:30:00</time></Open>\
        <Close><time>20260227-16:00:00</time></Close>\
        <Open><refDate>20260228</refDate><time";
    assert!(
        parse_schedule_response(truncated).is_none(),
        "a schedule cut short is not an answer",
    );
}

#[test]
fn parse_bar_response_basic() {
    let xml = r#"<ResultSetBar>
            <id>q1</id>
        <eoq>true</eoq>
        <tz>US/Eastern</tz>
        <Events>
            <Open><time>20260227-14:30:00</time></Open>
            <Bar>
                <time>20260227-14:30:00</time>
                <open>272.77</open>
                <close>269.47</close>
                <high>272.81</high>
                <low>269.2</low>
                <weightedAvg>270.998</weightedAvg>
                <volume>1411775</volume>
                <count>5165</count>
            </Bar>
            <Bar>
                <time>20260227-14:35:00</time>
                <open>269.48</open>
                <close>270.10</close>
                <high>270.50</high>
                <low>269.30</low>
                <weightedAvg>269.90</weightedAvg>
                <volume>500000</volume>
                <count>2000</count>
            </Bar>
            <Close><time>20260227-21:00:00</time></Close>
        </Events>
    </ResultSetBar>"#;

    let resp = parse_bar_response(xml).unwrap();
    assert_eq!(resp.query_id, "q1");
    assert_eq!(resp.timezone, "US/Eastern");
    assert!(resp.is_complete);
    assert_eq!(resp.bars.len(), 2);

    let bar = &resp.bars[0];
    assert_eq!(bar.time, "20260227-14:30:00");
    assert_eq!(bar.open, 272.77);
    assert_eq!(bar.high, 272.81);
    assert_eq!(bar.low, 269.2);
    assert_eq!(bar.close, 269.47);
    assert_eq!(bar.volume, 1411775);
    assert_eq!(bar.wap, 270.998);
    assert_eq!(bar.count, 5165);

    let bar2 = &resp.bars[1];
    assert_eq!(bar2.time, "20260227-14:35:00");
    assert_eq!(bar2.close, 270.10);
}

/// A trade count past what the callback carries is read as a bar that states
/// none, rather than reaching the caller as a negative one.
///
/// The count goes out to both surfaces as a signed 32-bit number. Held wider
/// and cast on the way, `3000000000` arrived as `-1294967296`: a bar made by
/// minus one and a quarter billion trades, which reads as data.
#[test]
fn a_trade_count_past_what_the_callback_carries_is_not_delivered_negative() {
    let xml = r#"<ResultSetBar>
        <id>q9</id>
        <eoq>true</eoq>
        <tz>US/Eastern</tz>
        <Events>
            <Bar>
                <time>20260227-14:30:00</time>
                <open>1.0</open><high>1.0</high><low>1.0</low><close>1.0</close>
                <volume>1</volume><weightedAvg>1.0</weightedAvg>
                <count>3000000000</count>
            </Bar>
        </Events>
    </ResultSetBar>"#;

    let resp = parse_bar_response(xml).expect("the bar still reaches the caller");
    assert_eq!(
        resp.bars[0].count, 0,
        "a count that will not fit is no count, and never a negative one",
    );
}

#[test]
fn parse_bar_response_incomplete() {
    let xml = r#"<ResultSetBar>
            <id>q2</id>
        <eoq>false</eoq>
        <tz>US/Eastern</tz>
        <Events>
            <Bar>
                <time>20260227-14:30:00</time>
                <open>100.0</open>
                <close>101.0</close>
                <high>102.0</high>
                <low>99.0</low>
                <volume>1000</volume>
                <count>10</count>
            </Bar>
        </Events>
    </ResultSetBar>"#;

    let resp = parse_bar_response(xml).unwrap();
    assert!(!resp.is_complete);
    assert_eq!(resp.bars.len(), 1);
}

/// A week and a month come back dated rather than timed.
///
/// The venue states `date` and `endDate` on a bar it aggregated where it
/// states `time` and `endTime` on everything shorter. A reader looking only
/// for `time` handed the caller a full candle with no date on it, which is
/// what a chart plots at the epoch.
#[test]
fn a_bar_the_venue_aggregated_is_read_from_the_date_it_states() {
    let xml = r#"<ResultSetBar>
        <id>q3</id>
        <eoq>true</eoq>
        <approxStep>1W</approxStep>
        <Events>
            <Bar>
                <date>20260309</date>
                <endDate>20260314</endDate>
                <open>666.39</open>
                <close>662.29</close>
                <high>683.36</high>
                <low>661.36</low>
                <weightedAvg>671.482</weightedAvg>
                <volume>339498148</volume>
                <count>3351333</count>
            </Bar>
        </Events>
    </ResultSetBar>"#;

    let resp = parse_bar_response(xml).unwrap();
    assert_eq!(resp.bars.len(), 1);
    assert_eq!(resp.bars[0].time, "20260309", "the week it covers");
    assert_eq!(resp.bars[0].close, 662.29);
    // And where it ends, which the start does not say. The last bar of a
    // series is normally partial, and this is what tells a finished week from
    // a running one.
    assert_eq!(resp.bars[0].end, "20260314", "and the day it closes on");
}

/// A bar shorter than a week states its end under the other spelling, and a
/// bar that states none carries none.
#[test]
fn a_bar_states_its_end_under_the_spelling_its_size_uses() {
    let bar = |inner: &str| {
        let xml = format!(
            "<ResultSetBar><id>q4</id><eoq>true</eoq><Events><Bar>{inner}             <open>1</open><close>2</close><high>3</high><low>1</low>             </Bar></Events></ResultSetBar>",
        );
        parse_bar_response(&xml).unwrap().bars.remove(0)
    };

    let minute = bar("<time>20260312-14:30:00</time><endTime>20260312-14:31:00</endTime>");
    assert_eq!(minute.time, "20260312-14:30:00");
    assert_eq!(minute.end, "20260312-14:31:00", "the same field, the other spelling");

    let silent = bar("<time>20260312-14:30:00</time>");
    assert_eq!(silent.end, "", "a bar the venue states no end for carries none");
}

#[test]
fn parse_bar_response_rejects_non_bar() {
    assert!(parse_bar_response("<ResultSetTickerId>...").is_none());
    assert!(parse_bar_response("not xml at all").is_none());
}

#[test]
fn parse_ticker_id() {
    let xml = r#"<ResultSetTickerId>
            <id>q1</id>
        <tickerId>42</tickerId>
    </ResultSetTickerId>"#;
    assert_eq!(super::parse_ticker_id(xml), Some("42".to_string()));
}

/// A tick-by-tick assignment names it differently, exactly as it arrives
/// from the server. Reading only `tickerId` left the subscription unbound
/// and no tick could be routed to it.
#[test]
fn parse_ticker_id_reads_the_tick_by_tick_spelling() {
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\t<ResultSetTickerId>\n\t\t                   <id>tbt_1</id>\n\t\t<rtTickerId>1</rtTickerId>\n\t\t                   <minTick>0.00005</minTick>\n\t\t<sizeMinTick>1</sizeMinTick>\n\t\t                   <eoq>false</eoq>\n\t</ResultSetTickerId>\n";
    assert_eq!(super::parse_ticker_id(xml), Some("1".to_string()));
}

#[test]
fn parse_ticker_id_rejects_other() {
    assert!(super::parse_ticker_id("<ResultSetBar>...</ResultSetBar>").is_none());
}

/// The contract's own security type and exchange have to reach the query.
/// Hardcoding them described a stock on SMART whatever was asked for, and
/// anything venue-specific is rejected with error 162.
#[test]
fn a_futures_query_carries_its_own_sec_type_and_exchange() {
    let req = HistoricalRequest {
        query_id: "hist_1".into(),
        con_id: 793356225,
        symbol: "MNQ".into(),
        sec_type: "FUT".into(),
        exchange: "CME".into(),
        data_type: BarDataType::Trades,
        end_time: String::new(),
        duration: "2 D".into(),
        bar_size: BarSize::Day1,
        use_rth: false,
        keep_up_to_date: false,
        include_expired: false,
    };
    let xml = build_query_xml(&req);
    assert!(xml.contains("<secType>FUT</secType>"), "got: {xml}");
    assert!(xml.contains("<exchange>CME</exchange>"), "got: {xml}");
    assert!(!xml.contains("BEST"), "a futures query must not be routed to BEST: {xml}");
}

/// SMART still maps to BEST, which is what stock callers relied on.
#[test]
fn a_smart_query_still_routes_to_best() {
    let req = HistoricalRequest {
        query_id: "hist_2".into(),
        con_id: 756733,
        symbol: "SPY".into(),
        sec_type: "CS".into(),
        exchange: "SMART".into(),
        data_type: BarDataType::Trades,
        end_time: String::new(),
        duration: "1 D".into(),
        bar_size: BarSize::Day1,
        use_rth: true,
        keep_up_to_date: false,
        include_expired: false,
    };
    let xml = build_query_xml(&req);
    assert!(xml.contains("<exchange>BEST</exchange>"), "got: {xml}");
    assert!(xml.contains("<secType>CS</secType>"), "got: {xml}");
}

#[test]
fn head_timestamp_xml_structure() {
    let req = HeadTimestampRequest {
        query_id: "tk_1".to_string(),
        con_id: 756733,
        sec_type: "STK".to_string(),
        exchange: "SMART".to_string(),
        data_type: "Last",
        use_rth: true,
        include_expired: false,
    };
    let xml = build_head_timestamp_xml(&req);
    assert!(xml.contains("<type>TickHeadTimeStamp</type>"));
    assert!(xml.contains("<contractID>756733</contractID>"));
    assert!(xml.contains("<exchange>BEST</exchange>")); // SMART→BEST
    assert!(xml.contains("<data>Last</data>"));
    assert!(xml.contains("<step>-1</step>"));
    assert!(xml.contains("<useRTH>true</useRTH>"));
    // Led by the query's own name, so two callers asking the same question of
    // the same contract are told apart. The venue answers a query named this
    // way: measured against a live historical connection, which is what a
    // change to a field the venue echoes back is worth waiting for.
    assert!(xml.contains("tk_1;;756733@BEST Last;;0;;true;;0;;U"), "{xml}");
}

#[test]
fn parse_head_timestamp_response_basic() {
    let xml = r#"<ResultSetHeadTimeStamp>
            <id>TickHeadClient1;;756733@BEST Last;;0;;true;;0;;U</id>
        <eoq>true</eoq>
        <headTS>19930129-09:00:00</headTS>
        <tz>US/Eastern</tz>
        <Events>
            <Open><time>19930129-14:30:00</time><refDate>19930129</refDate></Open>
            <Close><time>19930129-21:15:00</time></Close>
        </Events>
    </ResultSetHeadTimeStamp>"#;
    let resp = parse_head_timestamp_response(xml).unwrap();
    assert_eq!(resp.head_timestamp, "19930129-09:00:00");
    assert_eq!(resp.timezone, "US/Eastern");
}

#[test]
fn parse_head_timestamp_rejects_other() {
    assert!(parse_head_timestamp_response("<ResultSetBar>...</ResultSetBar>").is_none());
    assert!(parse_head_timestamp_response("not xml").is_none());
}

#[test]
fn build_schedule_xml_structure() {
    let xml = build_schedule_xml("sched_1", 756733, "20260312-19:34:06", "5 d", true, "CS", "BEST");
    assert!(xml.contains("<id>sched_1</id>"));
    assert!(xml.contains("<contractID>756733</contractID>"));
    assert!(xml.contains("<data>Schedule</data>"));
    assert!(xml.contains("<scheduleOnly>true</scheduleOnly>"));
    assert!(xml.contains("<step>1 day</step>"));
    assert!(xml.contains("<useRTH>true</useRTH>"));
    assert!(xml.contains("<timeLength>5 d</timeLength>"));
}

#[test]
fn parse_schedule_response_basic() {
    let xml = r#"<ResultSetSchedule>
            <id>sched_1</id>
        <eoq>true</eoq>
        <tz>US/Eastern</tz>
        <derivedStart>20260306-14:30:00</derivedStart>
        <Events>
            <Open><time>20260306-14:30:00</time><refDate>20260306</refDate></Open>
            <Close><time>20260306-21:00:00</time></Close>
            <Open><time>20260309-14:30:00</time><refDate>20260309</refDate></Open>
            <Close><time>20260309-21:00:00</time></Close>
        </Events>
    </ResultSetSchedule>"#;

    let resp = parse_schedule_response(xml).unwrap();
    assert_eq!(resp.query_id, "sched_1");
    assert_eq!(resp.timezone, "US/Eastern");
    assert_eq!(resp.start_date_time, "20260306-14:30:00");
    assert_eq!(resp.sessions.len(), 2);
    assert_eq!(resp.sessions[0].ref_date, "20260306");
    assert_eq!(resp.sessions[0].open_time, "20260306-14:30:00");
    assert_eq!(resp.sessions[0].close_time, "20260306-21:00:00");
    assert_eq!(resp.sessions[1].ref_date, "20260309");
}

#[test]
fn parse_schedule_response_rejects_other() {
    assert!(parse_schedule_response("<ResultSetBar>...</ResultSetBar>").is_none());
    assert!(parse_schedule_response("not xml").is_none());
}

#[test]
fn build_tick_query_xml_structure() {
    let xml = build_tick_query_xml("tk_1", 265598, "", "20260312-15:00:00", 100, "TRADES", true, "CS", "BEST", false, false);
    assert!(xml.contains("<id>tk_1</id>"));
    assert!(xml.contains("<type>TickData</type>"));
    assert!(xml.contains("<data>AllLast</data>"));
    assert!(xml.contains("<step>ticks</step>"));
    assert!(xml.contains("<timeLength>100 t</timeLength>"));
    assert!(xml.contains("<wholeDays>true</wholeDays>"));
}

#[test]
fn build_tick_query_xml_bid_ask() {
    let xml = build_tick_query_xml("tk_2", 265598, "", "20260312-15:00:00", 50, "BID_ASK", false, "CS", "BEST", false, false);
    assert!(xml.contains("<data>BidAsk</data>"));
    assert!(xml.contains("<useRTH>false</useRTH>"));
}

/// The size filter goes where a gateway writes it: on bid/ask when the caller
/// asks to leave out a change that moves only a size, on every midpoint query
/// whatever the caller asked, and never on trades. It follows the delay, the
/// field before it in the venue's query.
#[test]
fn a_tick_query_carries_the_size_filter_where_a_gateway_writes_it() {
    const FILTER: &str = "<filter><ignoreSize>true</ignoreSize></filter>";
    let q = |what: &str, ignore_size: bool| build_tick_query_xml(
        "tk", 265598, "", "20260312-15:00:00", 100, what, true, "CS", "BEST", false, ignore_size,
    );
    let asked = q("BID_ASK", true);
    assert!(asked.contains(&format!("<delay>auto</delay>{FILTER}</Query>")), "{asked}");
    assert!(!q("BID_ASK", false).contains("<filter>"));
    assert!(q("MIDPOINT", false).contains(FILTER));
    assert!(q("MIDPOINT", true).contains(FILTER));
    for what in ["TRADES", "AGGTRADES"] {
        assert!(!q(what, true).contains("<filter>"), "{what}");
    }
}

/// The query counts from the end it names. A start written into the end's
/// field asked for the ticks before the moment the caller wanted the ticks
/// after, and the answer looked right and covered the wrong side of the
/// clock.
#[test]
fn a_tick_request_names_one_end_and_counts_from_it() {
    use crate::control::historical::build_tick_query_xml;
    let q = |start: &str, end: &str| build_tick_query_xml(
        "tk", 265598, start, end, 100, "TRADES", true, "CS", "BEST", false, false,
    );
    // Either end is served, and the count says how far it reaches. A start
    // used to be refused before it was ever sent; the venue answers one with
    // a hundred and twenty-five ticks.
    let from_a_start = q("20260312-09:30:00", "");
    assert!(from_a_start.contains("<startTime>20260312-09:30:00</startTime>"));
    assert!(!from_a_start.contains("<endTime>"), "one end, not two");
    assert!(from_a_start.contains("<timeLength>100 t</timeLength>"));

    let to_an_end = q("", "20260312-15:00:00");
    assert!(to_an_end.contains("<endTime>20260312-15:00:00</endTime>"));
    assert!(!to_an_end.contains("<startTime>"));
    assert!(to_an_end.contains("<timeLength>100 t</timeLength>"));

    // Both named: a gateway navigates forward from the start and reads the
    // end only where the start is absent — the start wins, the end is
    // ignored, the count stands.
    let both = q("20260312-09:30:00", "20260312-15:00:00");
    assert!(both.contains("<startTime>20260312-09:30:00</startTime>"), "{both}");
    assert!(!both.contains("<endTime>"), "{both}");
    assert!(both.contains("<timeLength>100 t</timeLength>"), "{both}");

    // Neither named: the gateway still sends — its epoch-anchored forward
    // query — and relays the venue's own answer or refusal rather than
    // inventing a local one.
    let neither = q("", "");
    assert!(neither.contains("<startTime>19700101-00:00:00</startTime>"), "{neither}");
    assert!(!neither.contains("<endTime>"), "{neither}");
}

/// A historical size crosses as text because it can be a fraction of a share.
/// Read as a whole number, a fractional print was a print of nothing.
#[test]
fn a_fractional_historical_size_is_read_as_the_fraction_it_states() {
    let xml = r#"<ResultSetTick>
            <id>tk_frac</id>
        <eoq>true</eoq>
        <tz>US/Eastern</tz>
        <Events>
            <Tick><time>20260312-14:30:01</time><price>150.25</price><size>0.5</size><exch>NASDAQ</exch></Tick>
        </Events>
    </ResultSetTick>"#;
    let (_qid, data, _done) = parse_tick_response(xml, "TRADES").unwrap();
    match data {
        crate::types::HistoricalTickData::Last(ticks) => {
            assert_eq!(ticks.len(), 1);
            assert_eq!(ticks[0].size, 0.5, "half a share is half a share, not none");
        }
        _ => panic!("Expected Last variant"),
    }
}

#[test]
fn parse_tick_response_trades() {
    let xml = r#"<ResultSetTick>
            <id>tk_1</id>
        <eoq>true</eoq>
        <tz>US/Eastern</tz>
        <Events>
            <Tick><time>20260312-14:30:01</time><price>150.25</price><size>100</size><exch>NASDAQ</exch><cond></cond></Tick>
            <Tick><time>20260312-14:30:02</time><price>150.30</price><size>200</size><exch>NYSE</exch><cond>I</cond></Tick>
        </Events>
    </ResultSetTick>"#;
    let (qid, data, done) = parse_tick_response(xml, "TRADES").unwrap();
    assert_eq!(qid, "tk_1");
    assert!(done);
    match data {
        crate::types::HistoricalTickData::Last(ticks) => {
            assert_eq!(ticks.len(), 2);
            assert_eq!(ticks[0].price, 150.25);
            assert_eq!(ticks[0].size, 100.0);
            assert_eq!(ticks[0].exchange, "NASDAQ");
            assert_eq!(ticks[1].special_conditions, "I");
        }
        _ => panic!("Expected Last variant"),
    }
}

#[test]
fn parse_tick_response_bid_ask() {
    let xml = r#"<ResultSetTick>
            <id>tk_2</id>
        <eoq>true</eoq>
        <Events>
            <Tick><time>20260312-14:30:01</time><bidPrice>150.24</bidPrice><askPrice>150.26</askPrice><bidSize>500</bidSize><askSize>600</askSize></Tick>
        </Events>
    </ResultSetTick>"#;
    let (_, data, _) = parse_tick_response(xml, "BID_ASK").unwrap();
    match data {
        crate::types::HistoricalTickData::BidAsk(ticks) => {
            assert_eq!(ticks.len(), 1);
            assert_eq!(ticks[0].bid_price, 150.24);
            assert_eq!(ticks[0].ask_price, 150.26);
        }
        _ => panic!("Expected BidAsk variant"),
    }
}

/// A print series is read under the names the answer writes, and carries what
/// the venue marked each print with.
///
/// The answer names the venue and the note it makes about a print more shortly
/// than the fields they fill. Read for the field names, both came back empty on
/// every row — which is what an unattributed print looks like, so nothing said
/// they had never been read. The marks were read nowhere at all, so a print the
/// venue said was past its limit, or one it said the tape does not carry,
/// reached the caller as neither.
#[test]
fn a_print_series_is_read_under_the_names_the_answer_writes() {
    let xml = "<ResultSetTick><id>q</id><eoq>true</eoq><Events>\
         <Tick><time>20260312-14:30:01</time><price>150.25</price><size>100</size>\
         <exch>ARCA</exch><cond>odd lot</cond><flags>H U</flags></Tick>\
         </Events></ResultSetTick>";
    let (_, data, _) = parse_tick_response(xml, "TRADES").expect("the reply reads");
    let crate::types::HistoricalTickData::Last(prints) = data else {
        panic!("a series of prints");
    };
    assert_eq!(prints.len(), 1, "the row is read");
    assert_eq!(prints[0].exchange, "ARCA", "the venue that printed it");
    assert_eq!(prints[0].special_conditions, "odd lot", "and what it noted");
    assert!(prints[0].past_limit, "and that it was past the limit");
    assert!(prints[0].unreported, "and that the tape does not carry it");

    // A print the venue marked with nothing is marked with nothing.
    let plain = "<ResultSetTick><id>q</id><eoq>true</eoq><Events>\
         <Tick><time>20260312-14:30:02</time><price>150.25</price><size>100</size>\
         <exch>ARCA</exch></Tick></Events></ResultSetTick>";
    let (_, data, _) = parse_tick_response(plain, "TRADES").expect("the reply reads");
    let crate::types::HistoricalTickData::Last(prints) = data else { panic!("prints") };
    assert!(!prints[0].past_limit && !prints[0].unreported, "nothing is claimed for it");
}

/// A quote series is read under the names the answer writes.
///
/// The caller-facing message for one of these rows spells its fields the other
/// way round — priceBid beside bidPrice — and read for those spellings every
/// row of every series arrived with a bid, an ask and two sizes of nought: a
/// full run of quotes at zero, timed correctly and marked complete, that the
/// venue never stated. A row that states none of them is refused rather than
/// published as a quote at zero, the way a bar with no open, high, low or
/// close is.
#[test]
fn a_quote_series_is_read_under_the_names_the_answer_writes() {
    let xml = "<ResultSetTick><id>q</id><eoq>true</eoq><Events>\
         <Tick><time>20260312-14:30:01</time><bidPrice>150.24</bidPrice>\
         <askPrice>150.26</askPrice><bidSize>500</bidSize><askSize>600</askSize></Tick>\
         </Events></ResultSetTick>";
    let (_, data, _) = parse_tick_response(xml, "BID_ASK").expect("the reply reads");
    let crate::types::HistoricalTickData::BidAsk(quotes) = data else {
        panic!("a bid and ask series");
    };
    assert_eq!(quotes.len(), 1, "the row is read");
    assert_eq!(quotes[0].bid_price, 150.24, "the bid the venue stated");
    assert_eq!(quotes[0].ask_price, 150.26, "and the ask");
    assert_eq!(quotes[0].bid_size, 500.0, "and the size on each side");
    assert_eq!(quotes[0].ask_size, 600.0);

    // A row stating none of them is not a quote at zero.
    let empty = "<ResultSetTick><id>q</id><eoq>true</eoq><Events>\
         <Tick><time>20260312-14:30:01</time></Tick></Events></ResultSetTick>";
    assert!(
        parse_tick_response(empty, "BID_ASK").is_none(),
        "a series with nothing in its rows is refused rather than published as zeros",
    );
}

#[test]
fn parse_tick_response_midpoint() {
    let xml = r#"<ResultSetTick>
            <id>tk_3</id>
        <eoq>true</eoq>
        <Events>
            <Tick><time>20260312-14:30:01</time><price>150.25</price></Tick>
        </Events>
    </ResultSetTick>"#;
    let (_, data, _) = parse_tick_response(xml, "MIDPOINT").unwrap();
    match data {
        crate::types::HistoricalTickData::Midpoint(ticks) => {
            assert_eq!(ticks.len(), 1);
            assert_eq!(ticks[0].price, 150.25);
        }
        _ => panic!("Expected Midpoint variant"),
    }
}

#[test]
fn parse_tick_response_rejects_other() {
    assert!(parse_tick_response("<ResultSetBar>...</ResultSetBar>", "TRADES").is_none());
}

#[test]
fn build_realtime_bar_xml_structure() {
    let xml = build_realtime_bar_xml("rt_1", 265598, "TRADES", true, "CS", "BEST");
    // The contract is stated, not assumed: an FX pair is not a US stock.
    let fx = build_realtime_bar_xml("rt_2", 12087792, "MIDPOINT", false, "CASH", "IDEALPRO");
    assert!(fx.contains("<secType>CASH</secType>"), "{fx}");
    assert!(fx.contains("<exchange>IDEALPRO</exchange>"), "{fx}");
    assert!(fx.contains("<data>MidPoint</data>"), "{fx}");
    assert!(xml.contains("<id>rt_1</id>"));
    assert!(xml.contains("<type>BarData</type>"));
    assert!(xml.contains("<data>Last</data>"));
    assert!(xml.contains("<refresh>5 secs</refresh>"));
    assert!(xml.contains("<step>5 secs</step>"));
}

/// A payload the decoder can read: 4 bits of padding, a 1-bit flag and an
/// 8-bit count, a 31-bit low in ticks, then a 1-bit flag and a 16-bit volume.
/// Sixty-one bits, in eight bytes, written the way the reader takes them —
/// least significant bit first, with each four-byte group reversed, which is
/// its own inverse over two whole groups.
pub(crate) fn single_tick_payload(low_ticks: u32, volume: u32) -> Vec<u8> {
    let mut bits: Vec<u8> = Vec::new();
    let mut put = |value: u32, width: usize| {
        for i in 0..width {
            bits.push(((value >> i) & 1) as u8);
        }
    };
    put(0, 4);
    put(1, 1);
    put(1, 8);
    put(low_ticks, 31);
    put(1, 1);
    put(volume, 16);
    bits.resize(64, 0);

    let mut stream = [0u8; 8];
    for (at, bit) in bits.iter().enumerate() {
        stream[at / 8] |= bit << (at % 8);
    }
    stream.chunks(4).flat_map(|c| c.iter().rev().copied()).collect()
}

/// A bar's volume is a count of the increment the venue said this contract's
/// sizes move in, and its weighted average price is not.
///
/// The weighted sum is a raw wire figure weighted by those same counts, so the
/// counts cancel and the volume must not be put underneath it. Scaled on both
/// sides, the offset from the low moves by the reciprocal of the increment — a
/// contract counted in hundred-millionths reads a sixty-thousand bar at fifty
/// million. This walks a bar of several ticks, which is the only shape that
/// carries a weighted price at all.
#[test]
fn a_bars_volume_counts_the_increment_and_its_weighted_price_does_not() {
    // count > 1, so the bar carries deltas and a weighted sum.
    let mut bits: Vec<u8> = Vec::new();
    let put = |v: u32, w: usize, bits: &mut Vec<u8>| {
        for i in 0..w { bits.push(((v >> i) & 1) as u8); }
    };
    put(0, 4, &mut bits);        // padding
    put(1, 1, &mut bits);        // count width flag: 8 bits
    put(4, 8, &mut bits);        // count = 4
    put(1000, 31, &mut bits);    // low = 1000 ticks
    put(1, 1, &mut bits);        // delta width flag: 5 bits
    put(2, 5, &mut bits);        // open delta
    put(6, 5, &mut bits);        // high delta
    put(3, 5, &mut bits);        // close delta
    put(1, 1, &mut bits);        // wap width flag: 18 bits
    put(2000, 18, &mut bits);    // weighted sum
    put(1, 1, &mut bits);        // volume width flag: 16 bits
    put(500, 16, &mut bits);     // volume count = 500

    let mut packed = vec![0u8; bits.len().div_ceil(8)];
    for (i, &b) in bits.iter().enumerate() {
        if b == 1 { packed[i / 8] |= 1 << (i % 8); }
    }
    let mut payload = Vec::new();
    for chunk in packed.chunks(4) {
        let mut c = chunk.to_vec();
        c.reverse();
        payload.extend_from_slice(&c);
    }

    let min_tick = 0.01;
    let whole = decode_bar_payload(&payload, min_tick, 1.0).expect("decodes");
    let counted = decode_bar_payload(&payload, min_tick, 0.5).expect("decodes");

    assert_eq!(whole.count, 4, "the bar carries several ticks");
    assert_eq!(counted.volume, whole.volume * 0.5, "volume counts the increment");
    // 10.0 + 2000 * 0.01 / 500
    assert!((whole.wap - 10.04).abs() < 1e-9, "weighted price: {}", whole.wap);
    assert!(
        (counted.wap - whole.wap).abs() < 1e-9,
        "and it does not move with the increment: {} against {}",
        counted.wap, whole.wap,
    );
}


/// A trade count past the width every surface reports it in refuses the
/// payload.
///
/// The wide form takes the full thirty-two bits off the wire. Cast straight
/// through to the signed width the field is held in, a count above two billion
/// reached the caller as a bar made by minus two billion trades. Held at nought
/// instead it is worse: this count decides whether the delta and weighted
/// fields were written at all, so nought skips bits the sender wrote and every
/// field behind it is read from the wrong offset — and the cursor ends short of
/// the end rather than past it, so the overrun guard never fires.
#[test]
fn a_bar_count_past_what_the_field_carries_refuses_the_payload() {
    let mut bits: Vec<u8> = Vec::new();
    let put = |v: u32, w: usize, bits: &mut Vec<u8>| {
        for i in 0..w { bits.push(((v >> i) & 1) as u8); }
    };
    put(0, 4, &mut bits);            // padding
    put(0, 1, &mut bits);            // count width flag: the wide form
    put(u32::MAX, 32, &mut bits);    // a count above what the field carries
    put(1000, 31, &mut bits);        // low = 1000 ticks
    put(1, 1, &mut bits);            // delta width flag: 5 bits
    put(2, 5, &mut bits);            // open delta
    put(6, 5, &mut bits);            // high delta
    put(3, 5, &mut bits);            // close delta
    put(1, 1, &mut bits);            // wap width flag: 18 bits
    put(2000, 18, &mut bits);        // weighted sum
    put(1, 1, &mut bits);            // volume width flag: 16 bits
    put(500, 16, &mut bits);         // volume count

    let mut packed = vec![0u8; bits.len().div_ceil(8)];
    for (i, &b) in bits.iter().enumerate() {
        if b == 1 { packed[i / 8] |= 1 << (i % 8); }
    }
    let mut payload = Vec::new();
    for chunk in packed.chunks(4) {
        let mut c = chunk.to_vec();
        c.reverse();
        payload.extend_from_slice(&c);
    }

    // Refused, not read as a bar stating no count: the count decides whether
    // the fields behind it were written at all, so calling it nought reads
    // every one of them from the wrong offset and hands back a bar that is
    // complete, plausible and made up.
    assert!(
        decode_bar_payload(&payload, 0.01, 1.0).is_none(),
        "a count that will not fit refuses the payload",
    );
}

#[test]
fn decode_bar_payload_single_tick() {
    // Count of one, so the bar collapses to a single price: 15000 ticks of a
    // cent is 150.00, and the volume is stated in the narrow field.
    let bar = decode_bar_payload(&single_tick_payload(15_000, 100), 0.01, 1.0)
        .expect("a whole payload decodes");
    assert_eq!(bar.count, 1);
    assert!((bar.low - 150.00).abs() < 1e-9, "{bar:?}");
    assert!((bar.open - 150.00).abs() < 1e-9, "{bar:?}");
    assert!((bar.high - 150.00).abs() < 1e-9, "{bar:?}");
    assert!((bar.close - 150.00).abs() < 1e-9, "{bar:?}");
    assert!((bar.volume - 100.0).abs() < 1e-9, "{bar:?}");

    assert!(decode_bar_payload(&[], 0.01, 1.0).is_none());
}

/// A read past the end of the payload takes zeroes, and so does every field
/// after it. Unrecorded, a payload cut anywhere decodes into a bar of plausible
/// zeroes indistinguishable from one the venue sent.
#[test]
fn a_bar_payload_cut_short_is_not_decoded() {
    let whole = single_tick_payload(15_000, 100);
    for cut in 1..whole.len() {
        assert!(
            decode_bar_payload(&whole[..cut], 0.01, 1.0).is_none(),
            "{cut} of {} bytes decoded into a bar", whole.len(),
        );
    }
}
mod duration_spelling_tests {
    use super::super::{normalize_duration, validate_duration};

    /// The spelling each unit is taken in, measured against a live session in
    /// both cases. Getting one wrong is refused outright — "Invalid time
    /// length" — rather than corrected, so a caller asking for seconds or weeks
    /// got nothing while the same span in days was served. And a bare number
    /// is a number of seconds, as a gateway reads it.
    #[test]
    fn each_unit_is_spelled_the_way_the_venue_takes_it() {
        for (asked, sent) in [
            ("3600 S", "3600 S"), ("3600 s", "3600 S"),
            ("2 D", "2 d"), ("2 d", "2 d"),
            ("2 W", "2 W"), ("2 w", "2 W"),
            ("1 M", "1 m"), ("1 m", "1 m"),
            ("1 Y", "1 y"), ("1 y", "1 y"),
            ("60", "60 S"),
        ] {
            assert_eq!(normalize_duration(asked), sent, "asked for {asked}");
        }
    }

    /// The request path takes only the five units, and refuses what is not
    /// one of them before the venue is asked; a path of its own that sends a
    /// duration this does not validate still passes it through, because
    /// refusing it there would hide the venue's answer about what it accepts.
    #[test]
    fn an_unknown_unit_reaches_the_venue_unchanged() {
        assert_eq!(normalize_duration("5 Q"), "5 Q");
        assert_eq!(normalize_duration(""), "");
        assert!(validate_duration("5 Q").is_err());
    }

    /// What a gateway refuses of a duration before it asks the venue, in its
    /// words: nothing stated, a shape that is not an integer, a space and one
    /// of the five units, and a count outside its unit's range.
    #[test]
    fn a_duration_a_gateway_refuses_is_refused_in_its_words() {
        const FORMAT: &str = "When specifying a unit, historical data request duration format is \
                              integer{SPACE}unit (S|D|W|M|Y).";
        const INVALID: &str = "Historical data requested duration is invalid.";
        for (asked, why) in [
            ("", "Historical data request duration not specified."),
            // The count and the seconds' own floor.
            ("0 d", INVALID),
            ("10 S", INVALID),
            ("29 s", INVALID),
            ("0", INVALID),
            // Each unit's ceiling.
            ("90000 S", "Historical data request for greater than 86400 seconds rejected."),
            ("86401 S", "Historical data request for greater than 86400 seconds rejected."),
            ("400 d", "Historical data requests for durations longer than 365 days must be made in years."),
            ("366 d", "Historical data requests for durations longer than 365 days must be made in years."),
            ("60 W", "Historical data request for durations longer than 52 weeks must be made in years."),
            ("53 W", "Historical data request for durations longer than 52 weeks must be made in years."),
            ("24 m", "Historical data request for durations longer than 12 months must be made in years."),
            ("13 m", "Historical data request for durations longer than 12 months must be made in years."),
            // A unit outside the five is a miss of the shape, whatever the
            // venue would once have been asked to make of it.
            ("1 q", FORMAT),
            ("1 h", FORMAT),
            ("60 min", FORMAT),
            ("1 S ", FORMAT),
            ("S", FORMAT),
            ("1  S", FORMAT),
            ("1.5 S", FORMAT),
            ("-1 S", FORMAT),
            ("1 sec", FORMAT),
            // The shape is the gateway's whole-string match: a space short,
            // a space over, a letter in the count or nothing but a space is
            // a miss of it, not a length.
            ("60S", FORMAT),
            ("1x5 S", FORMAT),
            (" ", FORMAT),
            // A count past the width a gateway reads it in is refused, not
            // read around.
            ("99999999999 S", INVALID),
        ] {
            assert_eq!(validate_duration(asked).unwrap_err(), why, "asked for {asked:?}");
        }
    }

    /// What a gateway takes: the folded shape, at the edges of every range.
    #[test]
    fn a_duration_a_gateway_takes_is_taken() {
        for asked in [
            "60", "30 S", "30 s", "86400 S",
            "1 d", "1 D", "365 d",
            "1 W", "52 w",
            "1 m", "1 M", "12 m",
            "1 y", "1 Y", "100 y",
        ] {
            validate_duration(asked).unwrap_or_else(|why| panic!("{asked:?}: {why}"));
        }
    }
}
mod tick_data_type_tests {
    use super::super::tick_data_type;

    /// Each name a caller can use names a series the venue serves. A gateway
    /// reads a tick query's series against four names alone.
    #[test]
    fn each_known_name_maps_to_the_venues_own() {
        assert_eq!(tick_data_type("TRADES"), Ok("AllLast"));
        assert_eq!(tick_data_type("MIDPOINT"), Ok("MidPoint"));
        assert_eq!(tick_data_type("BID_ASK"), Ok("BidAsk"));
    }

    /// A name outside the four is refused rather than turned into trades, in
    /// a gateway's words — the option-exercise rate included, which is no
    /// name a gateway takes on any query. Turned into trades, a caller asking
    /// for another series was answered with a list of option prints and told
    /// nothing.
    #[test]
    fn a_name_it_does_not_know_is_refused() {
        for unknown in [
            "MIDPONT", "BID", "OptExInterestRate ", "anything",
            "OPTION_EXERCISE_INTEREST_RATE",
        ] {
            assert_eq!(
                tick_data_type(unknown),
                Err("Invalid source price".to_string()),
                "{unknown} was taken for trades",
            );
        }
        // An empty name has a refusal of its own.
        assert_eq!(
            tick_data_type(""),
            Err("Source price must not be empty".to_string()),
        );
    }
    // ── decode_bar_payload ───────────────────────────────────────────────
    //
    // The bit layout, stated as the wire states it. These were written against
    // a second reading of this format, which is gone: two implementations of
    // one decoder is how a sign-extension defect survived in the one the
    // engine calls while the one nothing called was right.

    #[test]
    fn decode_bar_single_trade() {
        // count=1: only low is meaningful; open=high=close=low, volume encoded
        // Build LSB-first bit stream, then reverse within 4-byte groups.
        //
        // Layout (LSB first within the reordered buffer):
        //   4 bits padding (0)
        //   1 bit count_flag = 1 (short count)
        //   8 bits count = 1
        //   31 bits low_ticks = 1000 (positive)
        //   (no delta fields when count==1)
        //   1 bit vol_flag = 1 (short volume)
        //   16 bits volume = 500
        //
        // Total: 4+1+8+31+1+16 = 61 bits, 8 bytes
        let min_tick = 0.01;
        let mut bits_lsb: Vec<u8> = Vec::new();

        // helper: push n bits from val LSB-first
        let push_lsb = |bits: &mut Vec<u8>, val: u64, n: usize| {
            for i in 0..n {
                bits.push(((val >> i) & 1) as u8);
            }
        };

        push_lsb(&mut bits_lsb, 0, 4);     // padding
        push_lsb(&mut bits_lsb, 1, 1);      // count_flag=1 (8-bit)
        push_lsb(&mut bits_lsb, 1, 8);      // count=1
        push_lsb(&mut bits_lsb, 1000, 31);  // low_ticks=1000
        // count==1: no delta/wap fields
        push_lsb(&mut bits_lsb, 1, 1);      // vol_flag=1 (16-bit)
        push_lsb(&mut bits_lsb, 500, 16);   // volume=500

        // Convert bit stream to bytes (LSB first)
        let byte_count = bits_lsb.len().div_ceil(8);
        let mut reordered = vec![0u8; byte_count];
        for (i, &b) in bits_lsb.iter().enumerate() {
            if b == 1 {
                reordered[i / 8] |= 1 << (i % 8);
            }
        }

        // Reverse within 4-byte groups to produce the wire payload
        let mut payload = Vec::new();
        for chunk in reordered.chunks(4) {
            let mut c = chunk.to_vec();
            c.reverse();
            payload.extend_from_slice(&c);
        }

        let bar = super::super::decode_bar_payload(&payload, min_tick, 1.0).unwrap();
        assert_eq!(bar.count, 1);
        assert!((bar.low - 10.0).abs() < 1e-9);    // 1000 * 0.01
        assert!((bar.open - bar.low).abs() < 1e-9);
        assert!((bar.high - bar.low).abs() < 1e-9);
        assert!((bar.close - bar.low).abs() < 1e-9);
        assert_eq!(bar.volume, 500.0);
    }

    #[test]
    fn decode_bar_multi_trade_short_deltas() {
        // count > 1 with narrow (5-bit) deltas
        let min_tick = 0.01;
        let mut bits_lsb: Vec<u8> = Vec::new();

        let push_lsb = |bits: &mut Vec<u8>, val: u64, n: usize| {
            for i in 0..n {
                bits.push(((val >> i) & 1) as u8);
            }
        };

        push_lsb(&mut bits_lsb, 0, 4);     // padding
        push_lsb(&mut bits_lsb, 1, 1);      // count_flag=1 (8-bit)
        push_lsb(&mut bits_lsb, 5, 8);      // count=5
        push_lsb(&mut bits_lsb, 2000, 31);  // low_ticks=2000

        // count > 1: delta fields
        push_lsb(&mut bits_lsb, 1, 1);      // width_flag=1 → 5-bit deltas
        push_lsb(&mut bits_lsb, 3, 5);      // delta_open=3
        push_lsb(&mut bits_lsb, 7, 5);      // delta_high=7
        push_lsb(&mut bits_lsb, 2, 5);      // delta_close=2

        // wap
        push_lsb(&mut bits_lsb, 1, 1);      // wap_flag=1 → 18-bit
        push_lsb(&mut bits_lsb, 100, 18);   // wap_sum=100

        // volume
        push_lsb(&mut bits_lsb, 1, 1);      // vol_flag=1 → 16-bit
        push_lsb(&mut bits_lsb, 1000, 16);  // volume=1000

        let byte_count = bits_lsb.len().div_ceil(8);
        let mut reordered = vec![0u8; byte_count];
        for (i, &b) in bits_lsb.iter().enumerate() {
            if b == 1 {
                reordered[i / 8] |= 1 << (i % 8);
            }
        }

        let mut payload = Vec::new();
        for chunk in reordered.chunks(4) {
            let mut c = chunk.to_vec();
            c.reverse();
            payload.extend_from_slice(&c);
        }

        let bar = super::super::decode_bar_payload(&payload, min_tick, 1.0).unwrap();
        assert_eq!(bar.count, 5);
        let low = 2000.0 * min_tick; // 20.00
        assert!((bar.low - low).abs() < 1e-9);
        assert!((bar.open - (low + 3.0 * min_tick)).abs() < 1e-9);
        assert!((bar.high - (low + 7.0 * min_tick)).abs() < 1e-9);
        assert!((bar.close - (low + 2.0 * min_tick)).abs() < 1e-9);
        assert_eq!(bar.volume, 1000.0);
        // wap = low + wap_sum * min_tick / volume = 20.0 + 100*0.01/1000
        let expected_wap = low + 100.0 * min_tick / 1000.0;
        assert!((bar.wap - expected_wap).abs() < 1e-9);
    }


    /// The unit prices are counted in is the venue's to state. Chosen here, a
    /// bar in anything that does not move in pennies is decoded wrong and
    /// handed over as though it were right.
    #[test]
    fn a_ticker_with_no_stated_unit_reads_no_bars() {
        let stated = "<ticker id=\"7\"><minTick>0.005</minTick></ticker>";
        assert_eq!(super::super::min_tick_of(stated, "7"), Some(0.005));

        let silent = "<ticker id=\"7\"></ticker>";
        assert_eq!(
            super::super::min_tick_of(silent, "7"), None,
            "no unit stated is no unit, not a penny",
        );
    }
}

/// One name per series, whichever request asks for it.
///
/// The midpoint was spelled three times in this file and two of them were
/// wrong. The venue takes only `MidPoint`, and answered the others with "no
/// historical market data" — which reads as a series that does not exist
/// rather than as a name it does not know, so nobody looked for a typo.
#[test]
fn every_request_asks_for_a_series_by_the_same_name() {
    for name in ["TRADES", "MIDPOINT", "BID", "ASK"] {
        let through_the_type = BarDataType::from_api_str(name)
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .as_str();
        let xml = build_realtime_bar_xml("q", 12087792, name, false, "CASH", "IDEALPRO");
        assert!(
            xml.contains(&format!("<data>{through_the_type}</data>")),
            "{name} goes out as something other than {through_the_type}: {xml}",
        );
    }
}

/// A settled contract is asked about as settled, on both query shapes.
///
/// Written as a flat `no`, a request for an expired future asked about a
/// contract that no longer exists and came back empty, whatever the caller's
/// own contract said.
#[test]
fn an_expired_contract_is_asked_about_as_expired() {
    let stated = |include_expired: bool| HistoricalRequest {
        query_id: "h_1".into(),
        con_id: 495512563,
        symbol: "ES".into(),
        sec_type: "FUT".into(),
        exchange: "CME".into(),
        data_type: BarDataType::Trades,
        end_time: "20260101-16:00:00".into(),
        duration: "1 D".into(),
        bar_size: BarSize::Hour1,
        use_rth: true,
        keep_up_to_date: false,
        include_expired,
    };
    assert!(build_query_xml(&stated(true)).contains("<expired>yes</expired>"));
    assert!(build_query_xml(&stated(false)).contains("<expired>no</expired>"));

    let ticks = |include_expired: bool| build_tick_query_xml(
        "tk_1", 495512563, "", "20260101-16:00:00", 100, "TRADES", true, "FUT", "CME", include_expired, false,
    );
    assert!(ticks(true).contains("<expired>yes</expired>"));
    assert!(ticks(false).contains("<expired>no</expired>"));
}

/// The two volatility series ask under the names the venue answers to.
///
/// They asked under `HV` and `IV`, which came in with the first import and are
/// names this venue does not know: measured against a paper session, both drew
/// no bars and the venue said "No historical market data ... NoType", where
/// trades on the same contract drew five. Under the names below the same two
/// requests drew four bars and five.
#[test]
fn the_volatility_series_ask_under_names_the_venue_answers_to() {
    assert_eq!(BarDataType::HistoricalVolatility.as_str(), "HistVol");
    assert_eq!(BarDataType::ImpliedVolatility.as_str(), "OptionImpliedVol");
}

/// The series the venue carries are asked for by the names it answers to.
///
/// Refused before they were sent, a caller could not ask for any of them. The
/// names were read off the venue and checked against it: on a paper session
/// FEE_RATE drew four bars for a stock and NAV_LAST five, while AGGTRADES and
/// YIELD_LAST drew none and the venue answered naming the series back —
/// "AggLast", "LastYield" — which is a contract with no such series, not a
/// name it does not know. A name it does not know comes back as "NoType",
/// which is what the two volatility series drew before they were corrected.
#[test]
fn the_series_the_venue_carries_are_asked_for_by_its_own_names() {
    for (asked, on_the_wire) in [
        ("AGGTRADES", "AggLast"),
        ("FEE_RATE", "FeeRate"),
        ("YIELD_BID", "BidYield"),
        ("YIELD_ASK", "AskYield"),
        ("YIELD_LAST", "LastYield"),
        ("YIELD_MARK", "MarkYield"),
        ("NAV_LAST", "NavLast"),
    ] {
        let read = BarDataType::from_api_str(asked)
            .unwrap_or_else(|why| panic!("{asked} is a series the venue carries: {why}"));
        assert_eq!(read.as_str(), on_the_wire, "{asked} goes out under the wrong name");
    }
}

/// The auction and option-chain series the venue serves and this client refused.
///
/// Five names the venue answers to on a bar query and on a head timestamp,
/// turned away here before anything was sent, so a caller asking for
/// historical option open interest — which the venue states no other way — got
/// a validation error instead of data. Their wire names are the venue's own
/// and do not follow from the caller's: both option-volume series are named
/// after the last trade rather than after the volume.
#[test]
fn the_auction_and_option_chain_series_are_asked_for_by_the_venues_own_names() {
    for (asked, on_the_wire) in [
        ("INDICATIVE_AUCTION_PRICE_SIZE", "AuctionIndicLast"),
        ("CALL_OPTION_OPEN_INTEREST", "CallOpenInterest"),
        ("PUT_OPTION_OPEN_INTEREST", "PutOpenInterest"),
        ("CALL_OPTION_VOLUME", "CallLast"),
        ("PUT_OPTION_VOLUME", "PutLast"),
    ] {
        let read = BarDataType::from_api_str(asked)
            .unwrap_or_else(|why| panic!("{asked} is a series the venue carries: {why}"));
        assert_eq!(read.as_str(), on_the_wire, "{asked} goes out under the wrong name");
        let req = HistoricalRequest {
            query_id: "q".to_string(),
            con_id: 265598,
            symbol: "AAPL".to_string(),
            sec_type: "CS".to_string(),
            exchange: "SMART".to_string(),
            data_type: read,
            end_time: "20260228-15:00:00".to_string(),
            duration: "1 d".to_string(),
            bar_size: BarSize::Day1,
            use_rth: true,
            keep_up_to_date: false,
            include_expired: false,
        };
        assert!(
            build_query_xml(&req).contains(&format!("<data>{on_the_wire}</data>")),
            "{asked} does not reach the bar query",
        );
        // The head timestamp reads the same table, so it takes them too.
        assert_eq!(head_timestamp_data_type(asked), Ok(on_the_wire), "{asked}");
    }
}

/// Aggregated trades are a tick series as well as a bar one.
///
/// The bar table has carried the name all along and the tick table refused it,
/// so the same series could be asked for as bars and not as ticks.
#[test]
fn aggregated_trades_can_be_asked_for_as_ticks() {
    assert_eq!(tick_data_type("AGGTRADES"), Ok("AggLast"));
    let xml = build_tick_query_xml(
        "tk_agg", 265598, "", "20260312-15:00:00", 10, "AGGTRADES", true, "CS", "BEST", false, false,
    );
    assert!(xml.contains("<data>AggLast</data>"), "{xml}");
}

/// The head timestamp reads the bar table, and a name outside it is refused
/// in a gateway's words — the option-exercise rate included, which a gateway
/// refuses on a head timestamp as it refuses it on a bar query: the name is
/// in neither table, and the venue is never reached to say what it holds.
#[test]
fn the_head_timestamp_reads_the_bar_table() {
    assert_eq!(
        head_timestamp_data_type("OPTION_EXERCISE_INTEREST_RATE"),
        Err("What to show value of OPTION_EXERCISE_INTEREST_RATE rejected.".to_string()),
    );
    assert!(BarDataType::from_api_str("OPTION_EXERCISE_INTEREST_RATE").is_err());
    // And everything the bar table takes, under the same names.
    for asked in ["TRADES", "MIDPOINT", "BID_ASK", "HISTORICAL_VOLATILITY"] {
        assert_eq!(
            head_timestamp_data_type(asked),
            BarDataType::from_api_str(asked).map(|s| s.as_str()),
            "{asked}",
        );
    }
    assert_eq!(
        head_timestamp_data_type("NONSENSE"),
        Err("What to show value of NONSENSE rejected.".to_string()),
    );
}

/// One name that is two series is asked as a gateway asks it: the bid side's
/// series, with the ask side added to the query beside it, each query under an
/// id of its own so the two answers come back told apart. Refused instead, a
/// legal series was unreachable through the drop-in.
#[test]
fn a_pair_name_goes_out_as_both_series() {
    let asked = BarDataType::from_api_str("YIELD_BID_ASK").expect("a name a gateway serves");
    assert_eq!(asked.as_str(), "BidYield");

    let req = HistoricalRequest {
        query_id: "q1".to_string(),
        con_id: 265598,
        symbol: "USGG10YR".to_string(),
        sec_type: "BOND".to_string(),
        exchange: "SMART".to_string(),
        data_type: asked,
        end_time: "20260228-15:00:00".to_string(),
        duration: "1 d".to_string(),
        bar_size: BarSize::Day1,
        use_rth: true,
        keep_up_to_date: false,
        include_expired: false,
    };
    let xml = build_query_xml(&req);
    assert_eq!(xml.matches("<Query>").count(), 2, "one query per side: {xml}");
    assert!(xml.contains("<data>BidYield</data>"), "{xml}");
    assert!(xml.contains("<data>AskYield</data>"), "{xml}");
    assert!(xml.contains("<id>q1</id>"), "{xml}");
    // The ask side is named apart by the series it asks, the way a gateway
    // names the two, so its answer is not filed as the bid side's.
    assert!(xml.contains("<id>q1;;USGG10YR@BEST AskYield</id>"), "{xml}");
    // Everything but the series and the id is the one ask, twice.
    assert_eq!(xml.matches("<contractID>265598</contractID>").count(), 2, "{xml}");
    assert_eq!(xml.matches("<step>1 day</step>").count(), 2, "{xml}");
    // A name that is one series still goes out as one query.
    let single = build_query_xml(&HistoricalRequest { data_type: BarDataType::YieldBid, ..req });
    assert_eq!(single.matches("<Query>").count(), 1, "{single}");
}

/// The adjusted series begins where the trades it is folded from begin, and a
/// gateway asks for the earliest trade to answer it.
#[test]
fn the_adjusted_series_begins_where_the_trades_do() {
    assert_eq!(head_timestamp_data_type("adjusted_last"), Ok("Last"));
    assert_eq!(head_timestamp_data_type("ADJUSTED_LAST"), head_timestamp_data_type("TRADES"));
}

/// The head-timestamp query states whether an expired contract is meant, as
/// the bar query does. A settled future asked about with the flag set was
/// asked about as a contract that no longer exists.
#[test]
fn the_head_timestamp_query_states_expired_as_the_bar_query_does() {
    let ask = |include_expired: bool| super::build_head_timestamp_xml(&super::HeadTimestampRequest {
        query_id: "tk_1".to_string(),
        con_id: 1, sec_type: "FUT".into(), exchange: "CME".into(), data_type: "TRADES", use_rth: false, include_expired,
    });
    assert!(ask(true).contains("<expired>yes</expired>"), "{}", ask(true));
    assert!(ask(false).contains("<expired>no</expired>"), "{}", ask(false));
}

/// The empty shape and the filled one are the same kind.
///
/// A series that cannot be read is still ended, and it is ended under the kind
/// the caller asked for. Ended under another, the answer goes to a callback
/// nobody is waiting on: the caller waits out its whole deadline for a
/// completion it was already sent.
#[test]
fn an_ended_series_is_the_kind_the_reply_would_have_been() {
    fn kind(data: &crate::types::HistoricalTickData) -> &'static str {
        match data {
            crate::types::HistoricalTickData::Midpoint(_) => "midpoint",
            crate::types::HistoricalTickData::Last(_) => "last",
            crate::types::HistoricalTickData::BidAsk(_) => "bidask",
        }
    }
    for what in ["TRADES", "MIDPOINT", "BID_ASK", "OPTION_EXERCISE_INTEREST_RATE"] {
        let xml = "<ResultSetTick><id>q</id><eoq>true</eoq><tz>UTC</tz><Events>\
             <Tick><time>20260714-13:30:00</time><price>1.0</price><size>1</size>\
             <bidPrice>1.0</bidPrice><askPrice>1.0</askPrice><bidSize>1</bidSize>\
             <askSize>1</askSize></Tick></Events></ResultSetTick>";
        let filled = parse_tick_response(xml, what).expect("the reply reads");
        assert_eq!(
            kind(&filled.1),
            kind(&no_ticks_of_the_kind(what)),
            "{what}: a series ended without being read goes to another callback",
        );
    }
}

/// One bar as the capture states it: time, open, high, low, close, volume, count.
type RtBar = (u32, f64, f64, f64, f64, u32, u32);

/// Known real-time bar captures off the wire, each raw frame with the bar it decodes to.
const RTBAR_CAPTURES: &[(&[u8], RtBar)] = &[
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x01\xf2\
          \x0c\x0c\xd6\x20\xda\xd3\x18\x30\x00\x02\x5b\x81\x3a",
        (1772552690, 262.90, 262.95, 262.89, 262.95, 603, 6),
    ),
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x01\xf7\
          \x0c\x0c\xd6\x03\x1b\xd1\x98\xb0\x00\x0f\x14\x86\x61",
        (1772552695, 262.93, 262.94, 262.88, 262.91, 3860, 24),
    ),
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x01\xfc\
          \x0c\x0c\xd6\xa5\x3c\xfe\x70\x30\x00\x14\x51\xa4\x9f",
        (1772552700, 262.94, 263.21, 262.93, 263.21, 5201, 41),
    ),
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x02\x01\
          \x0c\x0c\xd8\x06\xdd\x50\x66\x50\x00\x23\x2d\xb6\xf7",
        (1772552705, 263.22, 263.29, 263.04, 263.04, 9005, 54),
    ),
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x02\x06\
          \x0c\x0c\xd6\xc3\x5e\x90\x31\x30\x00\x10\x64\x8c\x2b",
        (1772552710, 263.03, 263.06, 262.94, 262.94, 4196, 26),
    ),
    (
        b"8=O\x019=0043\x0135=G\x01\
          \x00\xa8\x00\x00\x00\x01\x69\xa7\x02\x0b\
          \x0c\x0c\xd5\x83\x7f\x53\xa5\x30\x00\x15\x39\x8c\xa6",
        (1772552715, 262.93, 262.93, 262.84, 262.91, 5433, 27),
    ),
];

#[test]
fn rtbar_captures_decode_to_their_bars() {
    for (raw, (time, open, high, low, close, volume, count)) in RTBAR_CAPTURES {
        // Behind the header: a ticker id, the bar's time, then the payload's
        // length and the payload.
        let body = raw.strip_prefix(b"8=O\x019=0043\x0135=G\x01").expect("header");
        assert_eq!(u32::from_be_bytes(body[6..10].try_into().unwrap()), *time);
        let payload = &body[11..11 + body[10] as usize];
        let bar = decode_bar_payload(payload, 0.01, 1.0).expect("decodes");
        for (got, want) in [
            (bar.open, *open),
            (bar.high, *high),
            (bar.low, *low),
            (bar.close, *close),
        ] {
            assert!((got - want).abs() < 1e-6, "{time}: {got} != {want}");
        }
        assert_eq!(bar.volume, f64::from(*volume), "{time}");
        assert_eq!(bar.count, *count as i32, "{time}");
    }
}
