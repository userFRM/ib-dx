//! The security-definition farm.
//!
//! A connection of its own, stated by the venue at logon beside the two this
//! client already opened. It carries the corporate-events calendar, which the
//! trading connection answers `Request not supported` for — the sub-protocol
//! is served here and nowhere else.
//!
//! Absent where the venue stated no route. That is a fact about the session,
//! not a failure: everything else works without it, and a session that refused
//! to open because one farm was down would be worse than one that says which
//! farms it has.

use crate::engine::hot_loop::EventSink;
use std::time::Instant;

use crate::bridge::SharedState;
use crate::protocol::datetime::chrono_free_timestamp;
use crate::control::calendar as cal;
use crate::protocol::connection::{Connection, Frame};
use crate::protocol::fix;

use super::HeartbeatState;

#[derive(Default)]
pub struct SecDefState {
    /// The metadata request on the wire: the name this client gave it, which
    /// the answer echoes, and the caller waiting on it. A gateway holds one
    /// pending slot per kind, claimed as a request goes out and released when
    /// its answer, its refusal or the end of the connection settles it — and
    /// so does this client. A second request of a kind while its slot is held
    /// is refused under the gateway's number for the kind; nothing is queued
    /// behind the first, and the two kinds do not wait on each other.
    meta_pending: Option<(String, u32)>,
    /// The events request on the wire: its own slot.
    events_pending: Option<(String, u32)>,
    /// The metadata this session has been answered with. A gateway holds the
    /// calendar metadata in a cache written only by the metadata answer — no
    /// automatic fetch fills it — and an event request consults the cache
    /// before anything is built: empty, it is refused under the gateway's
    /// number rather than sent.
    meta_answered: Option<String>,
    /// Message types this connection has sent that nothing here reads, named
    /// once each. Reported where somebody looks, rather than dropped.
    unread: std::collections::HashSet<String>,
}

impl SecDefState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stop waiting on the calendar's metadata.
    ///
    /// Both cancels do the same thing, which is a gateway's quirk: either
    /// frees the metadata slot — silently, whatever is in it, and not keyed
    /// on the caller's number — and neither touches a pending events
    /// request, whose answer still reaches the caller. An answer for a
    /// freed metadata request is dropped when it arrives, as nothing here
    /// asked for it any more; the venue is never told to stop.
    pub(crate) fn cancel_calendar(&mut self) {
        self.meta_pending = None;
    }

    /// Ask what event types the calendar carries.
    pub(crate) fn send_calendar_meta_data_request(
        &mut self,
        req_id: u32,
        conn: &mut Option<Connection>,
        hb: &mut HeartbeatState,
        shared: &SharedState,
    ) {
        // A session a metadata answer already reached is completed from the
        // cache: the caller is answered at once with what that answer
        // carried, nothing goes on the wire and no slot is claimed. The
        // cache is consulted before the slot, as a gateway consults it.
        if let Some(cached) = &self.meta_answered {
            log::info!("Returning cached WSH meta data for req_id={req_id}");
            shared.reference.push_calendar_meta_data(req_id, cached.clone());
            return;
        }
        // One slot per kind: a second metadata request while the first is on
        // the wire is refused under the gateway's number and words.
        if self.meta_pending.is_some() {
            shared.reference.push_historical_error(
                req_id,
                cal::DUPLICATE_META_DATA_REQUEST,
                "Duplicate WSH meta data request.".to_string(),
            );
            return;
        }
        let Some(conn) = conn.as_mut() else {
            shared.reference.push_historical_error(
                req_id,
                cal::FAILED_META_DATA_REQUEST,
                "Failed to request WSH meta data.".to_string(),
            );
            return;
        };
        let key = format!("MetaDataRequest{req_id}");
        let ts = chrono_free_timestamp();
        let json = cal::meta_data_request();
        if let Err(e) = conn.send_fix(&[
            (fix::TAG_MSG_TYPE, "U"),
            (fix::TAG_SENDING_TIME, &ts),
            (6040, &cal::CALENDAR_SUB_PROTOCOL.to_string()),
            (cal::TAG_CALENDAR_KEY, &key),
            (cal::TAG_CALENDAR_REQUEST_KIND, &cal::CALENDAR_META_DATA.to_string()),
            (cal::TAG_CALENDAR_JSON, &json),
        ]) {
            // A gateway joins the fixed text and the failure directly, as
            // this one ends in a full stop.
            shared.reference.push_historical_error(
                req_id, cal::FAILED_META_DATA_REQUEST, format!("Failed to request WSH meta data.{e}"),
            );
            return;
        }
        hb.last_secdef_sent = Instant::now();
        self.meta_pending = Some((key, req_id));
        log::info!("Sent calendar metadata request: req_id={req_id}");
    }

    /// Ask the calendar for events.
    pub(crate) fn send_calendar_events_request(
        &mut self,
        req_id: u32,
        query: &crate::types::CalendarQuery,
        conn: &mut Option<Connection>,
        hb: &mut HeartbeatState,
        shared: &SharedState,
    ) {
        // The metadata answer is consulted before anything is built, as a
        // gateway consults it: a session no metadata answer reached has no
        // event path at all.
        if self.meta_answered.is_none() {
            shared.reference.push_historical_error(
                req_id,
                cal::META_DATA_NOT_REQUESTED,
                "WSH meta data not requested.".to_string(),
            );
            return;
        }
        // The request is built before the slot is claimed, as a gateway
        // builds it: a request that fails its own validation is refused
        // under that failure, not as a duplicate.
        let json = match cal::event_data_request(query) {
            Ok(json) => json,
            Err(why) => {
                shared.reference.push_historical_error(req_id, why.code, why.message);
                return;
            }
        };
        if self.events_pending.is_some() {
            shared.reference.push_historical_error(
                req_id,
                cal::DUPLICATE_EVENT_DATA_REQUEST,
                "Duplicate WSH event data request.".to_string(),
            );
            return;
        }
        let Some(conn) = conn.as_mut() else {
            shared.reference.push_historical_error(
                req_id,
                cal::FAILED_EVENT_DATA_REQUEST,
                "Failed to request WSH event data.".to_string(),
            );
            return;
        };
        let key = format!("CalendarRequest{req_id}");
        let ts = chrono_free_timestamp();
        if let Err(e) = conn.send_fix(&[
            (fix::TAG_MSG_TYPE, "U"),
            (fix::TAG_SENDING_TIME, &ts),
            (6040, &cal::CALENDAR_SUB_PROTOCOL.to_string()),
            (cal::TAG_CALENDAR_KEY, &key),
            (cal::TAG_CALENDAR_REQUEST_KIND, &cal::CALENDAR_EVENT_DATA.to_string()),
            (cal::TAG_CALENDAR_JSON, &json),
        ]) {
            // Joined directly, as the fixed text ends in a full stop.
            shared.reference.push_historical_error(
                req_id, cal::FAILED_EVENT_DATA_REQUEST, format!("Failed to request WSH event data.{e}"),
            );
            return;
        }
        hb.last_secdef_sent = Instant::now();
        self.events_pending = Some((key, req_id));
        log::info!("Sent calendar events request: req_id={req_id}");
    }

    /// Read whatever this connection has sent.
    pub(crate) fn poll(
        &mut self,
        conn: &mut Option<Connection>,
        shared: &SharedState,
        event_tx: &Option<EventSink>,
        hb: &mut HeartbeatState,
    ) {
        if let Err(lost) = self.read(conn, shared, event_tx, hb) {
            self.give_up_with(conn, shared, event_tx, &lost.to_string());
        }
    }

    /// Put the connection down and tell everyone waiting on it.
    ///
    /// Kept, every later request is sent into a socket that will never answer
    /// and waits indefinitely; put down, a caller is told at once that
    /// this session has no connection for the calendar — and the reconnect,
    /// which declines to build one while a connection is installed, is free to
    /// build another.
    pub(crate) fn give_up(&mut self, conn: &mut Option<Connection>, shared: &SharedState, event_tx: &Option<EventSink>) {
        self.give_up_with(conn, shared, event_tx, "it can no longer be written to");
    }

    /// Give it up because it stopped answering.
    pub(crate) fn give_up_silent(&mut self, conn: &mut Option<Connection>, shared: &SharedState, event_tx: &Option<EventSink>) {
        self.give_up_with(conn, shared, event_tx, "it stopped answering");
    }

    fn give_up_with(&mut self, conn: &mut Option<Connection>, shared: &SharedState, event_tx: &Option<EventSink>, why: &str) {
        *conn = None;
        // Announced as the other two data connections' losses are, under the
        // venue's own numbers for this one; its recovery is announced where
        // the socket is installed again.
        crate::engine::hot_loop::announce_venue_data(
            shared, event_tx, crate::bridge::VenueDataConnection::SecurityDefinition, false,
        );
        log::warn!("The calendar connection went ({why}); the requests it carried are reported as failed sends");
        // Each kind is reported under the gateway's number and words for
        // a request it could not send.
        let waiting = self.meta_pending.take().into_iter()
            .map(|(_, req_id)| (req_id, cal::FAILED_META_DATA_REQUEST, "Failed to request WSH meta data."))
            .chain(self.events_pending.take().into_iter()
                .map(|(_, req_id)| (req_id, cal::FAILED_EVENT_DATA_REQUEST, "Failed to request WSH event data.")));
        for (req_id, code, text) in waiting {
            shared.reference.push_historical_error(req_id, code, text.to_string());
        }
    }

    fn read(
        &mut self,
        conn: &mut Option<Connection>,
        shared: &SharedState,
        event_tx: &Option<EventSink>,
        hb: &mut HeartbeatState,
    ) -> Result<(), String> {
        let messages = match conn.as_mut() {
            None => return Ok(()),
            Some(conn) => {
                match conn.try_recv() {
                    Ok(0) if !conn.has_buffered_data() => return Ok(()),
                    Ok(0) => {}
                    Err(e) => {
                        log::error!("Security definition farm connection lost: {e}");
                        return Err(e.to_string());
                    }
                    Ok(_) => {
                        hb.last_secdef_recv = Instant::now();
                        // It answered, so whatever was asked of it is settled.
                        hb.pending_secdef_test = None;
                    }
                }
                let frames = conn.extract_frames();
                let mut msgs: Vec<Vec<u8>> = Vec::new();
                for frame in &frames {
                    match frame {
                        Frame::FixComp(raw) => {
                            let Some(unsigned) = conn.unsign(raw) else { continue };
                            match crate::protocol::fixcomp::fixcomp_decompress(&unsigned) {
                                Ok(inner) => msgs.extend(inner),
                                Err(e) => log::warn!(
                                    "Security definition farm: dropping a malformed frame: {e}",
                                ),
                            }
                        }
                        Frame::Fix(raw) | Frame::Binary(raw) => {
                            let Some(unsigned) = conn.unsign(raw) else { continue };
                            msgs.push(unsigned);
                        }
                        Frame::Control(_) => {}
                    }
                }
                msgs
            }
        };
        for msg in &messages {
            self.handle(msg, conn, shared, event_tx, hb);
        }
        Ok(())
    }

    fn handle(
        &mut self,
        msg: &[u8],
        conn: &mut Option<Connection>,
        shared: &SharedState,
        event_tx: &Option<EventSink>,
        hb: &mut HeartbeatState,
    ) {
        let parsed = fix::fix_parse(msg);
        let Some(msg_type) = parsed.get(&fix::TAG_MSG_TYPE) else { return };
        match msg_type.as_str() {
            "0" => {}
            // The venue asking whether this connection is still there. Left
            // unanswered it closes the connection, and the calendar with it.
            "1" => {
                let test_id = parsed.get(&fix::TAG_TEST_REQ_ID).cloned().unwrap_or_default();
                if let Some(conn) = conn.as_mut() {
                    let ts = chrono_free_timestamp();
                    let _ = conn.send_fix(&[
                        (fix::TAG_MSG_TYPE, fix::MSG_HEARTBEAT),
                        (fix::TAG_SENDING_TIME, &ts),
                        (fix::TAG_TEST_REQ_ID, &test_id),
                    ]);
                    hb.last_secdef_sent = Instant::now();
                }
            }
            "U" => match parsed.get(&6040).map(String::as_str) {
                Some(cal::CALENDAR_ANSWER) => {
                    self.deliver(&parsed, false, shared, event_tx);
                }
                Some(cal::CALENDAR_REFUSAL) => {
                    self.deliver(&parsed, true, shared, event_tx);
                }
                Some(other) if self.unread.insert(other.to_string()) => {
                    log::info!("Unread on the security definition farm: sub-protocol {other}");
                }
                Some(_) => {}
                None => {}
            },
            // A rejection carries no request id, so it belongs to whatever is
            // outstanding: with a request on the wire in either slot, both
            // callers are told rather than left waiting on an answer the
            // venue has already declined to give.
            "3" => {
                let said = parsed.get(&58).cloned().unwrap_or_else(|| "refused".to_string());
                let waiting = self.meta_pending.take().into_iter()
                    .chain(self.events_pending.take())
                    .map(|(_, req_id)| req_id)
                    .collect::<Vec<u32>>();
                if waiting.is_empty() {
                    log::warn!("Security definition farm refused something: {said}");
                    return;
                }
                for req_id in waiting {
                    shared.reference.push_historical_error(req_id, 321, said.clone());
                }
            }
            other => {
                if self.unread.insert(other.to_string()) {
                    log::info!("Unread on the security definition farm: type {other}");
                }
            }
        }
    }

    /// The calendar's answer, or its refusal, to the caller that asked.
    ///
    /// Matched by the name this client gave the request, which the answer
    /// echoes. An answer naming a request no slot holds — one already
    /// settled, or one its caller withdrew — reaches nobody, as a gateway
    /// ignores a response for a request its slot no longer holds.
    fn deliver(
        &mut self,
        parsed: &std::collections::HashMap<u32, String>,
        refused: bool,
        shared: &SharedState,
        event_tx: &Option<EventSink>,
    ) {
        let Some(key) = parsed.get(&cal::TAG_CALENDAR_KEY) else {
            log::warn!("A calendar answer arrived naming no request; dropping it");
            return;
        };
        let (req_id, is_meta) = if self.meta_pending.as_ref().is_some_and(|(k, _)| k == key) {
            let (_, req_id) = self.meta_pending.take().unwrap();
            (req_id, true)
        } else if self.events_pending.as_ref().is_some_and(|(k, _)| k == key) {
            let (_, req_id) = self.events_pending.take().unwrap();
            (req_id, false)
        } else {
            log::warn!("A calendar answer named '{key}', which nothing here asked for");
            return;
        };
        if refused {
            let said = parsed.get(&58).cloned().unwrap_or_else(|| "refused".to_string());
            shared.reference.push_historical_error(req_id, 321, said);
            return;
        }
        // The answer's payload. Absent is not empty: an answer carrying no
        // payload at all was delivered as a successful empty result, so a
        // caller could not tell a calendar with nothing in it from a reply
        // this client could not read.
        let Some(json) = parsed.get(&96).cloned() else {
            let told = format!(
                "the calendar answered request {req_id} with no payload, so there is \
                 nothing to read out of it"
            );
            log::warn!("{told}");
            shared.reference.push_historical_error(req_id, 321, told);
            return;
        };
        if is_meta {
            // The cache is written only here: by a metadata answer this
            // session asked for and got.
            self.meta_answered = Some(json.clone());
            shared.reference.push_calendar_meta_data(req_id, json.clone());
        } else {
            shared.reference.push_calendar_events(req_id, json.clone());
        }
        let _ = event_tx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sent(of: impl FnOnce(&mut SecDefState, &mut Option<Connection>, &mut HeartbeatState, &SharedState)) -> String {
        use std::io::Read;
        let (conn, mut peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        of(&mut state, &mut conn, &mut hb, &shared);
        let mut buf = [0u8; 8192];
        let n = peer.read(&mut buf).unwrap_or(0);
        String::from_utf8_lossy(&buf[..n]).replace('\u{1}', "|")
    }

    /// The calendar request goes out on this connection, whole.
    #[test]
    fn the_metadata_request_goes_out_whole() {
        let msg = sent(|state, conn, hb, shared| {
            state.send_calendar_meta_data_request(7, conn, hb, shared)
        });
        assert!(msg.contains("35=U|"), "{msg}");
        assert!(msg.contains("|6040=155|"), "the sub-protocol: {msg}");
        assert!(msg.contains("|8081=100|"), "which of the two: {msg}");
        assert!(msg.contains("|6556=MetaDataRequest7|"), "its own name: {msg}");
    }

    /// A session the venue stated no route for has no such connection, and a
    /// caller is told under the gateway's number and words that the
    /// request could not be sent, rather than left waiting.
    #[test]
    fn without_the_connection_a_caller_is_told() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        state.send_calendar_meta_data_request(7, &mut None, &mut HeartbeatState::new(), &shared);
        let told = shared.reference.drain_historical_errors();
        assert_eq!(told.len(), 1, "{told:?}");
        assert_eq!(
            (told[0].0, told[0].1), (7, cal::FAILED_META_DATA_REQUEST),
            "no connection is a failed send, not a malformed request: {told:?}",
        );
        assert_eq!(told[0].2, "Failed to request WSH meta data.", "{told:?}");
    }

    /// An event request that no metadata answer precedes this session is
    /// refused as a gateway refuses it, under its number and words, and
    /// nothing reaches the venue wire. A gateway holds the calendar metadata
    /// in a cache written only by the metadata answer: there is no automatic
    /// fetch, and an event request consults the cache before anything is
    /// built.
    #[test]
    fn events_before_any_metadata_answer_are_refused() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
        state.send_calendar_events_request(9, &query, &mut conn, &mut hb, &shared);
        let told = shared.reference.drain_historical_errors();
        assert_eq!(told.len(), 1, "{told:?}");
        assert_eq!((told[0].0, told[0].1), (9, cal::META_DATA_NOT_REQUESTED), "{told:?}");
        assert_eq!(told[0].2, "WSH meta data not requested.", "{told:?}");
        assert!(state.events_pending.is_none(), "nothing reaches the wire");

        // A metadata request answered this session is the only thing that
        // opens the event path; the same request then goes out.
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 7);
        state.send_calendar_events_request(9, &query, &mut conn, &mut hb, &shared);
        assert!(
            shared.reference.drain_historical_errors().is_empty(),
            "an answered metadata request opens the event path",
        );
        assert!(state.events_pending.is_some(), "and the request is on the wire");
    }

    /// A second metadata request in a session a metadata answer already
    /// reached is completed from the cache: nothing goes on the wire, no
    /// slot is claimed, and the caller is answered at once with what the
    /// first answer carried.
    #[test]
    fn a_second_metadata_request_is_answered_from_the_first() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 7);
        state.send_calendar_meta_data_request(8, &mut conn, &mut hb, &shared);
        assert!(state.meta_pending.is_none(), "nothing goes on the wire");
        assert!(
            shared.reference.drain_historical_errors().is_empty(),
            "a cached answer is not a refusal",
        );
        let pushed = shared.reference.drain_calendar_meta_data_for_dispatch();
        assert_eq!(pushed.len(), 1, "{pushed:?}");
        assert_eq!(pushed[0].0, 8, "{pushed:?}");
        assert_eq!(pushed[0].1, r#"{"meta_data":{"event_types":[]}}"#, "{pushed:?}");
    }

    /// A metadata request answered this session, which is the only way a
    /// gateway's calendar cache is written.
    fn answered_metadata(
        state: &mut SecDefState,
        conn: &mut Option<Connection>,
        hb: &mut HeartbeatState,
        shared: &SharedState,
        req_id: u32,
    ) {
        state.send_calendar_meta_data_request(req_id, conn, hb, shared);
        let mut reply = std::collections::HashMap::new();
        reply.insert(cal::TAG_CALENDAR_KEY, format!("MetaDataRequest{req_id}"));
        reply.insert(96, r#"{"meta_data":{"event_types":[]}}"#.to_string());
        state.deliver(&reply, false, shared, &None);
        shared.reference.drain_calendar_meta_data_for_dispatch();
    }

    /// The answer names the request this client gave it, which is the only
    /// thing that says which caller is waiting.
    #[test]
    fn an_answer_reaches_the_caller_that_asked() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let mut conn = Some(Connection::for_test().0);
        state.send_calendar_meta_data_request(7, &mut conn, &mut HeartbeatState::new(), &shared);

        let mut reply = std::collections::HashMap::new();
        reply.insert(cal::TAG_CALENDAR_KEY, "MetaDataRequest7".to_string());
        reply.insert(96, r#"{"meta_data":{"event_types":[]}}"#.to_string());
        state.deliver(&reply, false, &shared, &None);

        let answered = shared.reference.drain_calendar_meta_data_for_dispatch();
        assert_eq!(answered.len(), 1);
        assert_eq!(answered[0].0, 7);
    }

    /// A connection that has gone is put down rather than kept and written
    /// to. Kept, every later request went into a socket that would never
    /// answer and waited indefinitely.
    #[test]
    fn a_connection_that_went_is_put_down() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let (conn, peer) = Connection::for_test();
        let mut conn = Some(conn);
        state.send_calendar_meta_data_request(7, &mut conn, &mut HeartbeatState::new(), &shared);
        drop(peer);

        // Read until the dead socket is noticed.
        for _ in 0..4 {
            state.poll(&mut conn, &shared, &None, &mut HeartbeatState::new());
        }
        // Asserted rather than asked about: guarded by `if conn.is_none()`
        // this passed without checking anything in exactly the case it is
        // named for, where the dead connection is still installed.
        assert!(conn.is_none(), "the dead connection was put down");
        let told = shared.reference.drain_historical_errors();
        assert_eq!(told.len(), 1, "the caller was told rather than left waiting: {told:?}");
        assert_eq!((told[0].0, told[0].1), (7, cal::FAILED_META_DATA_REQUEST), "{told:?}");
        assert_eq!(told[0].2, "Failed to request WSH meta data.", "{told:?}");
    }

    /// The venue can answer either calendar query after 45 seconds while the
    /// connection keeps answering heartbeats; those callers are still waiting,
    /// each in its own slot — the two kinds do not serialize through one.
    #[test]
    fn calendar_answers_after_45_seconds_reach_the_callers() {
        let (mut venue, socket) = Connection::for_test();
        let mut conn = Some(Connection::new_raw(socket).unwrap());
        let mut hb = HeartbeatState::new();
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 5);
        // The send path completes a metadata request from the cache once an
        // answer filled it, so the slot is set the way the wire leaves it.
        state.meta_pending = Some(("MetaDataRequest7".to_string(), 7));
        let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
        state.send_calendar_events_request(9, &query, &mut conn, &mut hb, &shared);

        for _ in 0..9 {
            std::thread::sleep(std::time::Duration::from_secs(5));
            venue.send_fix(&[(fix::TAG_MSG_TYPE, fix::MSG_HEARTBEAT)]).unwrap();
            state.poll(&mut conn, &shared, &None, &mut hb);
            assert!(conn.is_some(), "the connection is still answering");
            let told = shared.reference.drain_historical_errors();
            assert!(told.is_empty(), "the venue has not refused either query: {told:?}");
            assert!(
                state.meta_pending.is_some() && state.events_pending.is_some(),
                "both callers are still waiting, each in its own slot",
            );
        }

        let metadata = r#"{"meta_data":{"event_types":[]}}"#;
        let events = r#"{"events":[]}"#;
        for (key, json) in [("MetaDataRequest7", metadata), ("CalendarRequest9", events)] {
            venue.send_fix(&[
                (fix::TAG_MSG_TYPE, "U"),
                (6040, cal::CALENDAR_ANSWER),
                (cal::TAG_CALENDAR_KEY, key),
                (96, json),
            ]).unwrap();
            for _ in 0..100 {
                state.poll(&mut conn, &shared, &None, &mut hb);
                if state.meta_pending.is_none() && state.events_pending.is_none() { break; }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        assert_eq!(shared.reference.drain_calendar_meta_data_for_dispatch(), vec![(7, metadata.to_string())]);
        assert_eq!(shared.reference.drain_calendar_events_for_dispatch(), vec![(9, events.to_string())]);
        assert!(shared.reference.drain_historical_errors().is_empty());
    }

    /// An answer carrying no payload is not an empty calendar. Delivered as
    /// one, a caller could not tell a calendar with nothing in it from a reply
    /// this client could not read.
    #[test]
    fn an_answer_with_no_payload_is_not_an_empty_calendar() {
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        state.send_calendar_meta_data_request(7, &mut conn, &mut hb, &shared);

        let soh = '\u{1}';
        let answer = format!(
            "35=U{soh}6040={}{soh}{}=MetaDataRequest7{soh}",
            cal::CALENDAR_ANSWER,
            cal::TAG_CALENDAR_KEY,
        );
        state.handle(answer.as_bytes(), &mut conn, &shared, &None, &mut hb);

        assert!(
            shared.reference.drain_calendar_meta_data_for_dispatch().is_empty(),
            "nothing readable arrived, so nothing is handed over as an answer",
        );
        let told = shared.reference.drain_historical_errors();
        assert!(
            told.iter().any(|(id, _, why)| *id == 7 && why.contains("no payload")),
            "and the caller is told why: {told:?}",
        );
    }

    /// A write that fails leaves the socket installed unless something gives
    /// it up. The read path does that when the connection goes, but a write
    /// can fail with no read error behind it — and the reconnect declines to
    /// build another while one is installed, so the calendar would stay on a
    /// socket nothing can be sent through for the life of the process.
    #[test]
    fn a_connection_that_cannot_be_written_to_is_given_up() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let sock = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (_peer, _) = listener.accept().unwrap();
        let mut conn = Some(Connection::new_raw(sock).unwrap());

        state.events_pending = Some((String::new(), 77));

        state.give_up(&mut conn, &shared, &None);

        assert!(conn.is_none(), "the connection is put down, so another can be built");
        assert!(state.events_pending.is_none(), "and nothing is left waiting on it");
        let told = shared.reference.drain_historical_errors();
        assert_eq!(told.len(), 1, "the caller is told rather than left waiting: {told:?}");
        assert_eq!((told[0].0, told[0].1), (77, cal::FAILED_EVENT_DATA_REQUEST), "{told:?}");
        assert_eq!(told[0].2, "Failed to request WSH event data.", "{told:?}");
    }

    /// A second request of a kind while the first is on the wire is refused
    /// under the gateway's number and words for the kind, and nothing is
    /// held behind the first to deliver a second answer later.
    #[test]
    fn a_second_request_of_a_kind_is_refused_while_the_first_is_on_the_wire() {
        for (is_meta, code, text) in [
            (true, cal::DUPLICATE_META_DATA_REQUEST, "Duplicate WSH meta data request."),
            (false, cal::DUPLICATE_EVENT_DATA_REQUEST, "Duplicate WSH event data request."),
        ] {
            let (conn, _peer) = Connection::for_test();
            let mut conn = Some(conn);
            let mut hb = HeartbeatState::new();
            let shared = SharedState::new();
            let mut state = SecDefState::new();
            let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
            if is_meta {
                state.send_calendar_meta_data_request(7, &mut conn, &mut hb, &shared);
                state.send_calendar_meta_data_request(8, &mut conn, &mut hb, &shared);
            } else {
                answered_metadata(&mut state, &mut conn, &mut hb, &shared, 5);
                state.send_calendar_events_request(7, &query, &mut conn, &mut hb, &shared);
                state.send_calendar_events_request(8, &query, &mut conn, &mut hb, &shared);
            }
            let told = shared.reference.drain_historical_errors();
            assert_eq!(told.len(), 1, "the second is refused, not held: {told:?}");
            assert_eq!((told[0].0, told[0].1), (8, code), "{told:?}");
            assert_eq!(told[0].2, text, "{told:?}");
        }
    }

    /// The two kinds do not wait on each other, and no rule is keyed on the
    /// caller's number: a metadata and an events request under one number are
    /// both on the wire at once, each answered under its own name.
    #[test]
    fn the_two_kinds_hold_the_wire_at_once() {
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 5);
        state.meta_pending = Some(("MetaDataRequest7".to_string(), 7));
        state.send_calendar_events_request(7, &query, &mut conn, &mut hb, &shared);
        assert!(
            shared.reference.drain_historical_errors().is_empty(),
            "neither kind refuses the other, and the number is not a rule",
        );
        assert!(state.meta_pending.is_some(), "the metadata request is on the wire");
        assert!(state.events_pending.is_some(), "and the events request beside it");
    }

    /// A reject carries no request id, so it belongs to whatever is
    /// outstanding: with a request in each slot, both callers are told
    /// rather than left waiting.
    #[test]
    fn a_reject_naming_no_request_refuses_whatever_is_outstanding() {
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 5);
        state.meta_pending = Some(("MetaDataRequest7".to_string(), 7));
        state.send_calendar_events_request(9, &query, &mut conn, &mut hb, &shared);

        let reject = fix::fix_build(&[(fix::TAG_MSG_TYPE, "3"), (58, "refused")], 1);
        state.handle(&reject, &mut conn, &shared, &None, &mut hb);
        let mut refused: Vec<u32> = shared.reference.drain_historical_errors().iter().map(|e| e.0).collect();
        refused.sort();
        assert_eq!(refused, [7, 9], "the reject belongs to both slots");
        assert!(
            state.meta_pending.is_none() && state.events_pending.is_none(),
            "and neither caller is left waiting",
        );
    }

    /// Both cancels do the same thing: the metadata slot is freed, whatever
    /// is in it, and a pending events request stays live — its answer still
    /// reaches the caller, while an answer for the freed metadata request is
    /// dropped as one nothing here asked for any more.
    #[test]
    fn a_cancel_frees_the_metadata_slot_and_leaves_the_events_alone() {
        let shared = SharedState::new();
        let mut state = SecDefState::new();
        let (conn, _peer) = Connection::for_test();
        let mut conn = Some(conn);
        let mut hb = HeartbeatState::new();
        answered_metadata(&mut state, &mut conn, &mut hb, &shared, 5);
        state.meta_pending = Some(("MetaDataRequest7".to_string(), 7));
        let query = crate::types::CalendarQuery { con_id: Some(265598), ..Default::default() };
        state.send_calendar_events_request(9, &query, &mut conn, &mut hb, &shared);

        state.cancel_calendar();
        assert!(state.meta_pending.is_none(), "the metadata slot is freed");
        assert!(state.events_pending.is_some(), "and the events request stays live");

        let mut meta_reply = std::collections::HashMap::new();
        meta_reply.insert(cal::TAG_CALENDAR_KEY, "MetaDataRequest7".to_string());
        meta_reply.insert(96, r#"{"meta_data":{"event_types":[]}}"#.to_string());
        state.deliver(&meta_reply, false, &shared, &None);
        assert!(
            shared.reference.drain_calendar_meta_data_for_dispatch().is_empty(),
            "an answer for the freed request reaches nobody",
        );

        let mut events_reply = std::collections::HashMap::new();
        events_reply.insert(cal::TAG_CALENDAR_KEY, "CalendarRequest9".to_string());
        events_reply.insert(96, r#"{"events":[]}"#.to_string());
        state.deliver(&events_reply, false, &shared, &None);
        let pushed = shared.reference.drain_calendar_events_for_dispatch();
        assert_eq!(pushed.len(), 1, "{pushed:?}");
        assert_eq!(pushed[0].0, 9, "the events answer still reaches its caller");
        assert!(shared.reference.drain_historical_errors().is_empty());
    }
}
