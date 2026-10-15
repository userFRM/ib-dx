# ib-dx: everything since ibx 0.7.1

One netted changelog, as if the work since DeepEntropy's last release (ibx 0.7.1, 25 July 2026) were a single squash merge. Each line states the end result on `main` (d7c9e85d, 15 October 2026); work that was redone later appears once, in its final form. Fixes are listed by the issue that tracked them.

- ibx 0.7.1 → ib-dx 0.1.0
- 1,999 commits, 557 files changed, 44k → 220k lines of Rust in `src/`
- 264 issues fixed on ib-dx, 10 closed without a fix of their own; 146 fork-era issue and PR references
- 4,818 offline tests + 182 that run against a paper session

## Identity, packaging and release

- Renamed: ibx → ibkr-dx → **ib-dx**. Crate `ib-dx` (library `ib_dx`), Python package `ib-dx` (`import ib_dx`), environment variables `IB_DX_*`, docs at userfrm.github.io/ib-dx. An independent project, not an official Interactive Brokers product.
- Version reset to **0.1.0**, prepared for the first public release on crates.io and PyPI. Minimum Rust 1.89, Python 3.11+, free-threaded 3.14t supported.
- Release workflow: a `v*` tag builds the crate, the sdist and 25 wheels (Linux x86-64 and arm64, macOS arm64 and Intel, Windows; Python 3.11–3.14 and 3.14t). Once the registry setup in #156 is done it publishes to crates.io with a token held by the `release` environment and to PyPI through trusted publishing; both publishing jobs need the owner's approval, and a manual run publishes nothing.
- Crate builds only the library (no unused cdylib for Rust dependents, no `ibx` executable: its quote demo is `examples/quote_printer.rs`); repository tools build only with `--features dev-tools` and are left out of the published packages. PyPI metadata carries `License-Expression: AGPL-3.0-only`; the publishing jobs run only actions pinned by commit.
- The ib_async integration lives in its own repository, ib_async-dx, which runs ib_async on this engine; the README points there.

### Fixed (2)

- #178 Release: the jobs that build and attach the published wheels run third-party actions by moving tag while the job that publishes them is pinned by commit
- #214 The one release job holding contents:write resolves actions/download-artifact by mutable tag, against the workflow's stated pinning policy

## Connection, login and session

- Announces the latest IB Gateway build at logon (now 10.51.1b), with the matching client string and launcher version; a nightly check fails when IB publishes a newer build.
- Name-service message version 52, with the session token's publish time on reconnect and on market-data logons.
- Second factor: IBKey code provider runs off the socket thread, the code is submitted as soon as it resolves and only while the socket is quiet; the authenticator-code second factor is supported; the second-factor timeout and cancel reach the caller.
- The server's Diffie-Hellman hello is validated instead of trusted; a frame failing signature verification is dropped; the known panic paths on gateway input are closed, and malformed-frame tests (truncated, mutated, invalid) guard them.
- Initial login tries the other built-in regional hosts when a default host does not answer. Reconnect tries every host this session was sent to, decides what to do from why the connection dropped, re-subscribes market data, and can be switched off (`reconnect_on_socket_err`).
- Connection loss is announced as a gateway does: 1100 when the trading connection goes, 1102 when it comes back, data-farm losses under their own notices; a program stays connected through a recoverable outage, and a data farm that cannot be recovered ends that feed without ending a healthy trading session.
- A placement written but not acknowledged when the connection drops is reconciled against the venue's working and completed orders after reconnect; one still unresolved is reported inactive, with 106 where a gateway states it; an unanswered cancel is re-sent while the order is still working.
- A session can be offered again after a restart: held in memory, written to disk only if asked. The server decides whether to take it; every restart observed so far has still asked for the ordinary login.
- Order ids persist per account and client id in a locked file keyed by a hash of the account, never the account number; next_valid_id is delivered at connect on both surfaces.
- A caller is told when another session already holds the account; a session can be opened that refuses to trade (read-only).
- Confirmed shutdown: disconnect says goodbye, drains, and reports how the engine ended.
- No part of the login name or session token appears in any log line.

### Fixed (9)

- #31 Act on the newer logon feature tokens
- #61 Session connect: state message version 52 and the token publish time
- #63 Connection loss: error 1100 is sent only after three failed reconnect attempts
- #111 A system clock before 1970 panics every connect and every log record instead of degrading
- #129 Second connect() raises RuntimeError instead of reporting error 501 'Already connected.' on the wrapper
- #213 A log directory that cannot be created panics the host process from import and from connect instead of degrading
- #235 A code provider waiting on a human makes connect never return: the second-factor timeout and the cancel flag never reach the caller
- #242 The nightly build tracker reports a newer published gateway build than this client announces
- #252 connect() carries a host string the wire cannot hold into the login, where the reference client refuses it with 579

## Orders

- All 159 Order fields accounted for and documented: 128 sent, 23 taken and not sent exactly as a gateway sends nothing for them, 6 filled in by the venue on the way back, `transmit` acted on, and 1 not carried that says so on itself.
- Order types and attributes added or fixed on the wire: relative, pegged-to-benchmark with each reference field in its own tag, passive relative, peg-to-best, stop-protect, minute-based peg time in force, scale/ladder orders and hedges, discretion, position effect, combination legs with their own prices, conditional orders with only the flags each condition carries, conditionsIncludeOvernight. Some order types a gateway takes are not placed yet and are refused by name under 387 (volatility orders and the volatility pegs, LMT/REL + MKT and REL + LMT, trailing MIT/LIT, RPI, PEG STK and a few others).
- Attached orders: bracket families linked by parentId and transmit, preset attached orders constructed from the account's order presets, versioned client order ids, the originator tags on every leg, and wide order ids carried through. Group and model percentage allocation for preset children is still open (#26): state explicit quantities there.
- Cash-quantity orders checked, sized and sent as a gateway does, including fund orders, local refusals with a gateway's codes, and the size estimate shown on the order.
- Algorithm orders checked against the venue's own algorithm definitions, fetched once per session; algo-stage refusals 439/442/443/10205 with a gateway's texts.
- Replacing an order: brackets, algo and conditional orders, stops (the trigger moves) and the outside-RTH flag are carried; a modify the replace cannot express is refused instead of cancelling the order.
- Order status and openOrder published as a gateway publishes them: cumulative fill, held print, whyHeld (child, locate, trigger), PendingSubmit/ApiPending, unset prices as unset, the venue's message beside a status as error 399, time-in-force names decoded.
- Order callbacks follow the placing client, as a gateway's do (client 0 also hears orders placed outside the API, and the master API client hears every client's); an order a session is told of but did not place comes with its openOrder, under its API number and its parent. Client 0 binds orders placed outside the API, with `orderBound` once the venue accepts. A cancel names this client's orders only, by the number this client knows each by. reqOpenOrders and reqCompletedOrders are scoped and dated as a gateway does.
- Global cancel waits for the venue's working orders to be named, cancels those and the ones placed since, drops held local placements, and says so when naming was incomplete. Cancels take `OrderCancel` (operator and manual-entry indicators); exercise carries manual time, customer account and professional-customer fields.
- Exercise and lapse an option position; what-if previews state the commission and the regular margin impact.
- Contracts named on every order path (index, bond, warrant, CFD, crypto, futures by maturity tag); orders refused for an unknown account, out-of-range or non-finite values, and quantities the wire cannot carry. Prices go out as stated: a price off the contract's tick grid is the venue's to refuse.

### Fixed (57)

- #1 Bracket placed with parentId and transmit is not linked: a filled child leaves its sibling working
- #2 A refused order modification reports the order inactive while the venue keeps it working
- #3 Order ids can repeat across sessions on the same account
- #4 Order on a contract whose quote was just cancelled cannot be modified (105)
- #5 Modifying an order with useAutoPriceForHedge drops the attribute and the venue refuses the revision
- #25 Order refusal codes and echoed fields that differ from a gateway
- #30 conditionsIncludeOvernight (level 226), and the highest level in the docs
- #37 Attached order child ids are 32-bit while next_valid_id can be wider
- #39 Order-id file stores the account number in plain text
- #43 Conditional orders: cancel-on-condition and ignore-RTH flags sent on each other's tags
- #54 open_order states permId 0 in the session that placed the order
- #55 A refused cancel reports the orders Inactive and drops them while the venue still holds them
- #59 Cash-quantity orders: order size is not derived from the cash amount
- #64 Cancel refused by the venue: reply to the program differs from a gateway's
- #66 Orders: an unset price is sent as zero
- #69 Errors: an order refusal carries the venue's time
- #70 Orders: a placement written but not acknowledged when the connection drops
- #71 Algo orders: definitions are not fetched, and the algo-stage refusals are missing
- #72 openOrder: the order's total quantity is taken from every execution report
- #75 Cancel after a refused modification states another quantity than a gateway's
- #76 orderStatus remaining after a cancel is 0, where a gateway states the order's size
- #77 reqOpenOrders answers with every client's orders, not the asking client's
- #78 An order a report states inactive is published as Inactive, where a gateway keeps its prior status
- #80 An order is restated to the program on every report of it, not only where its status moved
- #90 A cancel written and unanswered at a drop is never re-sent; the order keeps working
- #91 orderStatus states lastFillPrice 0.0 on every non-fill report; a gateway states the held value
- #92 Order callbacks reach the program for every client's order; a gateway sends them to the order's client and a master alone
- #93 reqCompletedOrders asks yesterday-to-tomorrow on the UTC clock; a gateway asks the session's day
- #112 Completed-order archive keeps every completion for the session's life and is cloned whole on every answer
- #113 Shared completed-order queue accumulates every live completion forever when the caller never asks
- #114 Generated-family map leaks an entry per member of every family withdrawn before it was sent
- #116 The published list of accepted order types names types the placement check refuses, and demotes two canonical names to aliases
- #134 A child order naming a parent the gateway does not know is sent to the venue; a gateway refuses it locally with 135 "Can't find order with id =<parentId>" and sends nothing
- #135 Cancelling a held (transmit=false) order tells the caller nothing: no orderStatus and no error; a gateway answers with its cancelled-status and 202 "Order Canceled - reason:Order was discarded"
- #136 Cancelling an order this session saw finish answers 161 with an invented text; the gateway's API cancel handler answers 10148 "OrderId %s that needs to be cancelled can not be cancelled, state: %s."
- #137 Cancelling an id nothing is working answers 135 with invented words; the gateway answers 10147 "OrderId %s that needs to be cancelled is not found."
- #141 orderStatus never states whyHeld; a gateway states "child", "locate" or "trigger" on every held (PreSubmitted) order
- #142 what-if reply margins are read from the last occurrence of each tag, not the Regular-impact group the gateway publishes; warning text is read from a tag the gateway's what-if parser never reads
- #143 The engine bracket hardcodes its two exit legs to GTC (and the entry to DAY); no reference states either, so exits outlive the session they were placed in
- #152 The recovery's completed-orders question states no account; a gateway scopes it to the account the naming carried
- #165 Cash-quantity sizing divides by zero when the mark rounds to a zero mantissa at 8 places, ending the session in debug and release alike
- #179 OrderState.completedTime is published as the raw venue UTC stamp where a gateway publishes a zoned local-time rendering
- #180 reqOpenOrders and reqAllOpenOrders never append the ApiPending status tail for placements still in processing
- #197 A cash quantity large enough to overflow the size computation sends a wrong size on tag 38 in release and closes the session in debug
- #198 A contract description stating one minimum size of zero kills the session in debug builds when one cash-quantity order is sized
- #202 openOrder and orderStatus state lmtPrice and auxPrice as 0.0 when the order carries no limit or stop, where the gateway states Double.MAX_VALUE
- #215 Order reports publish raw time-in-force characters where a gateway publishes decoded names
- #216 Bracket legs go out without the originator tags every other placement states
- #234 A margin condition percent outside the gateway's (0, 100) range is converted and sent to the venue where a gateway refuses the order with 320
- #257 An order stating a contract identifier of an unknown source kind is sent, where the details surface refuses it with 321
- #261 The order builder's naming gates read the unset id marker as a stated identifier
- #266 An order on an option or a future under the unset identifier marker skips the identity check zero gets
- #271 The two clients disagree on what a combination leg must state as its id
- #296 openOrder is not sent for an order this session is told of but did not place
- #299 Recovered orders can report the wrong Continuous Update value
- #300 Returned orders can report the wrong Sweep to Fill value
- #301 Returned orders can report incorrect Hidden, Auto Cancel Parent and Imbalance Only values

## Executions, commissions, positions and account

- Commission reports carry realized P&L, yield and redemption date, are told once per day key across reconnects, and follow their fill to its client.
- Executions recognised however many came before; a trade bust or correction is published as one signed execution, as a gateway does; execution and completed times follow the datetime-format setting.
- Execution queries include executions the venue restates at login, also after a process restart, without announcing them as new fills; `last_n_days` and `specific_dates` filters apply on the session timezone, with 320 for an invalid date.
- Positions and portfolio: daily P&L figures, holdings the account does not hold itself, positions on any account the login may act for, named portfolio contracts, and account values for each request that asked.
- Account figures delivered as the venue states them, with ledger figures kept apart from base-currency ones.

### Fixed (13)

- #9 AccruedCash in the base currency: account and ledger figures collide
- #10 Empty AccountCode after a fill reaches account update subscribers
- #52 An earlier order's replayed report can finish or fill a new order under the same id
- #53 A non-zero client's fill is reported as client 0's
- #65 Executions: a reconnect may deliver an execution again after more than 1024 fills in a day
- #158 Execution.time is published as the venue's raw UTC tag-60 string; a gateway publishes it reformatted in the operator's timezone with a zone suffix
- #174 Execution times ignore the datetime-format settings a gateway honours
- #175 An unreadable execution stamp is passed through raw instead of falling back
- #201 One trade bust or correction is announced to callers as one more positive fill, where the gateway publishes one synthetic execution carrying signed shares and one delta-derived price
- #240 updatePortfolio delivers a nameless contract where the gateway names the holding from the position row's fields
- #243 An execution timestamp that only positional decoding accepts is published in the legacy date-plus-time shape regardless of the datetime-format setting
- #250 Completed-order and execution contracts state no multiplier or trading class when this session never fetched the definition
- #253 A request for the advisor's allocation profiles is forwarded here, where the reference client refuses it with 585 and sends nothing

## Market data

- Many of the venue's additional series read: the numbered figure tables kept whole, company figures and ratios, IPO estimate, average option volume, imbalance, the trade tape, odd lots, the venue's mark, halt and trading status, issuer details, short-sale restriction.
- Market data types 1–4 with the venue's frozen and delayed-frozen machinery: a closed market serves the frozen quote, delayed fallback for a login without live data, and the gateway's refusal codes (354, 10089) when a quote is refused.
- tickReqParams sent once, with the first data served; extra price precision and odd-lot quotes asked for where the logon offers them.
- Quote subscriptions held to the line allowance the login states (40 when none is stated), refused under 101 beyond it; callers sharing a contract share one line.
- More than one caller can watch the same contract; tick-by-tick (all last, bid/ask, midpoint) read record by record even when streams share a frame, carrying `number_of_ticks` and `ignore_size`; real-time bars with several bars per frame; depth with smart and regular books held apart, the requested row window kept as levels move, and a book reset before the rebuilt levels after a reconnect.
- Request validation as a gateway: duplicate ids (322 with a gateway's text), empty exchange, BAG without legs, unknown generic ticks, snapshot rate, currencies outside the session's set, unknown market data type; generic tick 787 refused under 10367 where the session was not offered odd lots. The fundamentals series (258) is forwarded: a gateway's feature gate on it is documented as not reproduced.
- Option snapshots collect the computations available and end after eleven seconds; regulatory snapshots are decoded, with the permission check, timeout and missing-exchange-map error documented as differing from a gateway.

### Fixed (39)

- #8 tickReqParams is delivered twice per market data request
- #11 req_mkt_data_ex takes no market data options list
- #19 A refusal naming both parts of a market data request reaches nobody
- #22 Delayed market data type gives nothing on a contract served live
- #23 cancel_tick_by_tick_data does not stop the stream
- #24 Market data and reference: smaller differences from a gateway
- #35 Tick-by-tick on a contract named by description drops its expiry
- #58 Tick-by-tick: records of two streams in one frame are decoded as the first stream's kind
- #60 Market data: ask for extra price precision and odd-lot quotes when the venue offers them
- #62 tickReqParams is sent at the subscription's acknowledgement, before any data
- #68 Market data: request numbers wider than 32 bits and cancels under reserved numbers
- #81 reqMktData with an empty exchange is served; a gateway refuses it with 321 "Please enter exchange"
- #82 reqMktData snapshot with a generic tick list is served; a gateway refuses it
- #83 reqMarketDataType with an unknown number is silently ignored; a gateway refuses under -1 with 321
- #84 An incorrect generic tick list is trimmed and sent; a gateway refuses the whole request with 321
- #85 Locally raised 321/322 refusals carry the bare reason; a gateway wraps them
- #86 reqMktData on a BAG without combo legs is served; a gateway refuses it, and stated legs are dropped
- #87 reqTickByTickData: BAG/PDC not refused, and an incorrect type is refused in invented words
- #89 Generic ticks 787 and 258 are forwarded where a gateway refuses them (10367/10358) (787 now refused; the 258 gate is documented as not reproduced)
- #117 Duplicate reqMktData ticker id is refused under bare 102 with invented prose; a gateway refuses it under 322 with its catalog text
- #118 Duplicate reqMktDepth ticker id is refused under bare 102 with invented prose; a gateway refuses it under 322 with its catalog text
- #119 A second book of the other depth mode under one reqId is refused locally; a gateway holds smart and regular books apart and lets both run
- #120 Cancel of an unknown id carries the right code but invented text; a gateway's is the catalog sentence with the id joined on
- #121 Duplicate tick-by-tick and real-time-bar ids carry the right bare 102 but invented prose; a gateway's text is exactly "Duplicate ticker id"
- #146 Generic tick request ids the gateway resolves to a different venue series are subscribed verbatim: 220, 59, 47, 91, 104, 428 (and 221) ask the venue for series no gateway ever asks for
- #147 The creditman mark price (venue series 221) is delivered as tickGeneric 37 (MARK_PRICE); a gateway delivers that series only as 78 (CREDITMAN_MARK_PRICE), and its 37 only ever carries the Pl Price of series 232
- #159 reqRealTimeBars validates whatToShow against the historical table, with the historical refusal text; a gateway takes exactly {TRADES, AGGTRADES, MIDPOINT, BID, ASK} case-sensitively and refuses the rest with one catalog sentence
- #162 DepthMktDataDescription.aggGroup defaults to 0 where both references default it to UNSET_INTEGER
- #163 The tick-type enum lacks the reference's NOT_USED member and carries FUNDAMENTAL_RATIOS, which the reference does not have
- #181 A snapshot reqMktData burst beyond 100 per second is served where a gateway refuses the excess with 321
- #203 One refused real-time-bar or tick-by-tick venue query is relayed as 162 with the historical prefix, where a gateway states 420 for bars and 10189 for tick-by-tick
- #204 The gateway's request-time tick-by-tick refusals 10189 and 10190 do not exist: one unsupported type/security-type pair is not refused with its sentence, and the request cap is not enforced
- #205 reqTickByTickData and reqRealTimeBars admit every contract and subscribe at the venue, missing the gateway's request-time API-access refusal (10089/354)
- #206 Tick-by-tick stream sharing and cancellation are keyed on (instrument, kind) without ignore_size, so cancelling one stream leaves its differing sibling's venue query running for the session
- #207 A market-data or depth request stating a currency outside the session's set is served instead of refused with 406
- #208 A NEWS market-data request naming no registered provider is subscribed instead of refused with 321 listing the valid sources
- #217 A streaming price tick never delivers the paired size tick the reference always emits, and sizes arrive grouped after all prices
- #246 A whitespace-only exchange passes as stated on the quote request and is refused as empty on the historical, ticks and depth requests
- #251 A depth subscription the venue refuses is reported under 354 in words a gateway does not use

## Options

- The venue's own option model read, with bid/ask/last/model computations (10–13, 80–83) built from the venue's model data, currency rate curves, dividend schedules with their ex-dates, and the underlying's own quote and mark. Documented differences remain: frozen model ticks use the live model, and VIX-style, quanto, forex-index and futures-style premium cases.
- Standalone option prices and greeks; implied volatility and option price calculations from the venue's model; option chains per trading class.

### Fixed (4)

- #28 Option computation ticks: bid, ask and last computations, dividend value and attribute
- #46 Option ticks 10/11/12: no greeks, because the rate curve is never read
- #47 Option chains give every trading class the union of all classes' expiries
- #49 Frozen market data: option computation ticks carry 0.0 as the option price

## Historical data, news, fundamentals, scanners and calendar

- Historical bars: every series the venue carries for stocks and funds asked along the contract's id history, ADJUSTED_LAST adjusted for dividends, keepUpToDate continuing from the last historical bar (a daily bar keeps the history's session end, then rolls at midnight UTC), crypto aggregated trades mapped as a gateway maps them, a request that can be withdrawn while still being served, at most 50 queries in flight.
- Validation as a gateway: bar sizes, durations (seven refusals), date formats, limited history (166), whatToShow per request type, endDateTime parsing (10314), duplicate ids, head timestamp and histogram rules.
- Historical ticks (start and end), head timestamps, histograms, corporate actions read rather than discarded.
- News: providers, historical headlines with their window, articles, bulletins kept across calls; fundamentals forwarded as a gateway forwards them.
- Scanners: subscription filters carried to the wire, permission notices (490/492), denied codes refused (10359/10360), concurrency capped by the logon's message rate, market name on each row.
- Corporate-events calendar: metadata, event data, two request slots and the gateway's 10278–10309 refusals; the venue's clock answerable in milliseconds.

### Fixed (60)

- #12 cancel_historical_data can withdraw another request's corporate-actions query
- #33 keepUpToDate daily bars do not roll to the next session
- #34 Second 5-second-bar caller on a contract receives nothing
- #48 Real-time bar frames with several bars: only the first is read
- #51 Adjusted bars do not follow a contract's id history across an id change
- #57 ADJUSTED_LAST bars are not adjusted for dividends
- #67 Historical data: request lifecycle differs from a gateway's in three places
- #73 Historical data: a live bar stream refused while its history is still arriving
- #74 Historical data: disconnect notice, request number ranges and duplicate-number texts
- #79 Historical-data service errors: text prefix, and the codes for histogram and tick errors
- #94 reqHistoricalData YIELD_BID_ASK is refused locally; a gateway serves it
- #95 Bar-size table diverges: "3 months"/"1 year" refused, aliases and any casing let through
- #96 reqHistoricalData duration is never validated: seven refusals and the bare-number reading are missing
- #97 Duplicate reqId refusals go out under 386/385; a gateway sends 322 with the wrapped text
- #98 formatDate outside {1,2,3} is accepted; a gateway refuses it
- #99 keepUpToDate with "1 secs" bars is refused locally; a gateway has no bar-size refusal for live updates
- #100 No cap on in-flight API historical queries; a gateway runs at most 50 and queues the rest
- #101 No cap on simultaneous scanner subscriptions; a gateway refuses beyond (max message rate)/10
- #102 Historical ticks/head timestamps accept whatToShow values a gateway refuses
- #103 cancelHistoricalData for an unknown reqId: code 366 is right, text is invented
- #122 The gateway's limited-history refusal (code 166) is missing: queries reaching back past the contract's window are served instead of refused
- #123 reqFundamentalData refuses unknown report types locally, in invented words under 321, where a gateway forwards them and relays the venue's answer under 430
- #124 A venue QueryError naming a historical-news, news-article or fundamentals query matches no pending list: the request leaks until the connection drops and the caller is told nothing
- #125 For a crypto contract the gateway swaps TRADES for AGGTRADES and advises under 10299; ibkr-dx serves the unaggregated series and says nothing
- #126 Historical bar volume parsed as i64 — fractional venue volume silently becomes 0
- #144 reqHistoricalTicks with both start and end dates is refused locally, but the gateway accepts it and answers forward from the start
- #145 A second reqHistoricalTicks under a reqId already in flight is sent instead of being refused with 102 Duplicate ticker id
- #149 Unparseable historical endDateTime and tick start/end datetimes are forwarded raw instead of refused at intake with 10314
- #154 A second reqHeadTimestamp under a live reqId is not refused; a gateway refuses it, and the repo later reports a spurious timeout
- #161 ScanData has no marketName field or constructor parameter
- #167 A crypto bar query stating aggregated trades is served aggregated where a gateway maps it back to raw trades
- #170 A scanner row's contract details never state the market name the row carried
- #182 reqHistoricalTicks with numberOfTicks 0 is sent to the venue where a gateway refuses it with 321 "Number of ticks must be > 0"
- #183 reqHistoricalTicks on a contract stating no exchange is served where a gateway refuses it with 321 "Exchange must not be empty"
- #184 reqHistoricalTicks on a BAG or PDC contract is served where a gateway refuses it with 321 "Combo types are not supported"
- #185 reqHeadTimeStamp on a contract stating no exchange is served where a gateway refuses it with 321 "Please enter exchange"
- #186 reqHistoricalData on a contract stating no exchange is served where a gateway refuses it with 321 "Please enter exchange"
- #187 reqHistoricalData on a BAG contract stating no combo legs is served where a gateway refuses it with 321 before sending
- #188 reqHistogramData periods a gateway refuses with 321 "Invalid time period" are converted case-insensitively or passed through to the venue
- #189 reqHistoricalNews with totalResults of zero or below is served where a gateway refuses it with 321 "Total results must be > 0"
- #190 reqHistoricalNews naming a provider the session is not subscribed to is forwarded where a gateway refuses it with 321 "Not subscribed for '<code>' provider"
- #192 Scanner codes and filters denied to the session are subscribed anyway where a gateway refuses them with 10359 and 10360 before sending
- #209 reqNewsBulletins(false) destroys the queued day's bulletins where a gateway keeps its cache and a true call replays it
- #218 A calendar event request before any metadata answer is forwarded where a gateway refuses it with 10282
- #219 A second calendar request is queued or refused under 102 where a gateway holds two slots and refuses with 10278 or 10281
- #220 An event data request naming both a contract and a filter, or naming neither, is forwarded where a gateway refuses it with 10309
- #221 A contract-named event data request carries the caller's fill flags where a gateway forces all three false
- #222 An event data request naming contract 0 or a negative contract is refused or rewritten where a gateway forwards it
- #223 Cancelling a calendar request invents a 300 refusal for unknown numbers and withdraws by req id of either kind
- #224 A second metadata request re-asks the venue where a gateway answers it at once without new venue traffic
- #225 News-calendar requests from a session without the news entitlement are forwarded or refused with an invented 504 text where a gateway refuses with 10276 and 10277
- #228 reqNewsArticle forwards an unsubscribed provider or an empty article id where a gateway refuses both at the door with 321
- #241 A bar query kept up to date streams the series the caller stated where a gateway streams the mapped one
- #245 A historical request invalid in two ways hears the exchange sentence here, where a gateway states the combo-legs sentence
- #247 The histogram period parser diverges from a gateway's reading on three edge inputs
- #249 A histogram request under the number of a live head-timestamp request is served, and a head timestamp under a live histogram is refused — both the reverse of a gateway
- #264 A scanner subscription field carrying None runs as its default where the reference client refuses the send with 524
- #267 A scanner subscription filter carrying None is left out of the scan where the reference client refuses the send with 524
- #268 A scanner subscription passed as None runs as an all-default scan where the reference client refuses the send with 524
- #270 A historical news request with a time bound this client cannot read is refused with a sentence a gateway does not have

## Contracts and reference data

- Contracts looked up by id alone for every security type but BAG (refused under 321, as a gateway does), by description when no id is given, and refused before sending as a gateway refuses them (identifier, security type, non-ASCII symbol, expiry and strike, zero strike, unset id marker).
- Every field and every identifier a definition states is kept (CUSIP and others), with market rules (price and size tables), bond terms, fund charges, the economic-value rule, the underlying and the time trading stops. Trading and liquid hours are rendered on the timezone named beside them.
- A large lookup times out on silence rather than on total time, and a timeout states how many definitions arrived.
- Display groups served; the Nasdaq listing named as a gateway names it (ISLAND when granted and enabled).

### Fixed (17)

- #20 Lookup by contract id alone fails for options, CFDs, bonds and FX
- #29 Check lastTradeDateOrContractMonth before sending (10372)
- #88 reqMarketRule refuses rules not learned this session, in words a gateway does not use
- #150 Contract.strike conflates "unset" with "zero": zero-strike options lose their strike on every request, and the reference's own unset marker goes on the wire as a 309-digit strike
- #160 Contract-description refusals a gateway makes before sending (identifier required, valid security type, non-ASCII symbol, option/futures expiry-strike-right) are absent from every market-data surface
- #171 A contract description stating the maximum integer identifier is not read as unstated
- #172 A security identifier with no source type, or a mismatched pair, is not refused
- #173 An option description stating a zero strike is accepted where a gateway refuses it by default
- #176 A contract identifier of 2147483647 reaches the engine as a real id on by-id requests
- #177 An unknown identifier source type is silently dropped on the contract-details lookup
- #191 reqContractDetails on a BAG contract is served where a gateway refuses it with 321 "'BAG' isn't supported for contract data request"
- #226 reqMatchingSymbols refusals carry one invented 321 sentence where a gateway has two exact validation texts, and the feature-off refusal is missing
- #227 An unanswered matching-symbols request is reported as -1 "timed out" after 12s where a gateway reports 10159 "Failed to request matching symbols:Error sending message to a CCP." after about 1s
- #232 ContractDetails.evMultiplier arrives as a float where the reference decodes and defaults it as an int
- #233 ContractDetails.secIdList reads as an empty list where the reference keeps None until the wire states entries
- #256 An id left at the unset marker is asked on the wire as 2147483647 on the corporate-actions, option-chain and market-data surfaces
- #262 A NEWS contract-details request naming no registered source is sent instead of refused with 321 listing the valid sources

## Rust surface

- A TWS API shaped EClient/EWrapper: all 87 calls of the canonical list and 88 of its 90 callbacks (the two others declared as stubs), with Wrapper::error carrying its error time and req_ids taking its count.
- Requests and cancels return nothing and take no wrapper: they return once admitted, and every answer, refusal, connection notice and order report arrives in session order through `process_msgs`. `ErrorOrigin` and `error_from` say whether an error belongs to a request, an order, the session or the client, beside the ordinary `error`.
- Sizes, positions and volumes delivered as a Decimal with the reference's unset marker; quote reads return a QuoteSnapshot with prices as numbers; `account()` returns `None` until the account download is complete.
- Calls that answer (contracts, bars, option chains, holdings, account values, scans, schedules), optional events on a bounded standard-library channel, `wait_for_data()` and `on_data()` to wake a caller's event loop, and `traffic()` for the session's byte and message totals.

### Fixed (9)

- #38 Rust: next_valid_id not delivered at connect
- #50 Rust: connect takes no client id, so every order is client 0's
- #153 The Rust EClient's reqMktData takes no option list where the reference request takes one, so a Rust caller's options can never reach the wire
- #212 The Rust EWrapper trait delivers sizes, positions and bar volume as f64 where the reference wrapper signatures declare Decimal
- #237 The beyond-the-api table's Rust column states a wrong return type and a same-name guarantee two rows contradict
- #244 Request methods beyond the reference client's surface are offered without the check that removed cancel_order_by_perm_id
- #260 A bracket placed through the native convenience call ignores an unknown contract identifier source kind where the order path refuses it with 321
- #263 The native advisor calls forward the retired allocation profiles and refuse an unknown partition in words of their own
- #272 Rust quick start and EClient: helpers outside the TWS API surface, and quotes as fixed-point integers

## Python surface

- The reference client's module layout, names and spellings, so a callback-based ibapi program moves with an import change (TWSSyncWrapper is not provided): module aliases, constructors taking the reference spelling, keyword arguments, enum and printing helpers, record classes printing as ibapi prints them, order conditions with their text.
- Callbacks delivered in the shape the program's wrapper declares (3, 4 or 5 argument error; commissionReport or commissionAndFeesReport).
- Sizes delivered as Decimal where the reference delivers Decimal; unset values kept unset; None handled as the reference handles it at send time.
- Calls that answer on the Python surface too, and a caller's own refusal can be put into the ordered stream.
- The reference client's 10.50 protobuf request family (the `…ProtoBuf` calls) routed to the text requests, with the exceptions documented (configuration calls answer 10357, `useProtoBuf` is False); `setOptionalCapabilities` and the client-side `validate*` helpers a moved program calls.
- Blocking login, waits and shutdown release the GIL; KeyboardInterrupt and SystemExit raised in a callback reach the caller; clients take part in cyclic garbage collection.

### Fixed (29)

- #21 Python delivers contractDetailsEnd for the client's own internal lookups
- #41 Python surface: a way to put a caller's own refusal into the ordered stream
- #42 Python: dir() does not list the TWS API attribute names on ContractDetails, Execution, OrderState
- #104 TickTypeEnum answers only toStr; ibapi 9.81's to_str raises AttributeError and kills the session
- #105 OrderState lacks the former commission field names both reference client generations declare
- #109 No notebook runs as written on a clean install: python-dotenv and pandas are named nowhere
- #127 Every delivered size/quantity is float or int where the current reference API delivers Decimal
- #128 OrderState.suggestedSize and OrderAllocation quantities are Python strings where the reference delivers Decimal
- #130 Order-condition classes default isMore/price/time/percent to fabricated values the reference refuses to send
- #131 One-rename port fails: ibkr_dx.errors, ibkr_dx.order_status, ibkr_dx.sync_wrapper modules and user-facing utils names do not exist
- #132 EClient lacks setOptionalCapabilities and the reference's validate* helpers
- #133 floatMaxString/decimalMaxString format differently from the reference helpers programs import
- #138 The promised one-rename port fails on ibapi's commission_report module: neither ibkr_dx.commission_report nor CommissionReport exists
- #148 Every keyword call into the EWrapper base methods raises TypeError — the defaults publish `_`-prefixed parameter names, and twelve callbacks' names differ from the reference's words; a raise inside a callback closes the session
- #151 getEnumTypeName answers the attribute name where the reference client answers the member's own string: NoneItem for None, Assigned for 'Assigned '
- #155 Order totalQuantity/filledQuantity, ContractDetails minSize/sizeIncrement/suggestedSizeIncrement and the portfolio position arrive as floats where the reference decodes Decimal
- #157 The first notebook a reader opens names the wrong project and contradicts the product's own headline: ibx connects directly to the brokerage gateway
- #168 The Enum class module the reference's constant classes are built on does not import
- #169 The reference's field-decoding helper is missing from the utils module
- #193 The 10.50 protobuf surface is absent and undocumented — printProtoSingleLine, useProtoBuf, the req*ProtoBuf family and the *ProtoBuf wrapper stubs
- #194 Printing a record class gives an invented string where ibapi prints its own, Execution has no string at all, and an unset Order double prints as 309 digits
- #195 The six order-condition classes print an invented repr instead of ibapi's condition text
- #196 PriceCondition has no TriggerMethodEnum, so the official sample's price-condition construction raises AttributeError
- #210 ContractDetails minAlgoSize, lastPricePrecision and lastSizePrecision are floats marked unset with f64::MAX where ibapi holds Decimals at UNSET_DECIMAL
- #211 The python BarData carries public timezone and end attributes the reference class does not define
- #229 The condition classes define neither valueToString nor setValueFromString, so programs that render or rebuild condition trigger text crash on every condition kind
- #230 OrderCondition takes no constructor argument and is no base of the condition classes, so reference code instantiating it or isinstance-checking against it fails
- #231 validate_attached_orders_parameters rejects the reference's attachedOrders keyword because its parameter is named order
- #254 The data classes reject None at construction where the reference client stores it and fails at send time on the error callback

## Gateway settings

- Gateway settings on the client: read-only API, timezone and locale, date-time format for timestamps, which execution reports are downloaded, the older Nasdaq spelling, reconnect on socket error, the announced build, the master API client id (`master_client_id`), whether FX positions and today's flat opening positions are sent (`include_fx_positions`, `send_zero_positions_for_todays_opening_only`), the announced network identity (the connection's own interface, or `mac_address`/`IB_DX_MAC` and `lan_ip`/`IB_DX_IP`), whether the venue's working orders are downloaded at connection (`download_open_orders_on_connection`; what the download names is folded into it, not told as live callbacks), and whether orders bound to client 0 take negative ids (`use_negative_auto_range`, on by default). `setServerLogLevel` changes this client's logger.
- Still to do, tracked in #273 (#278, #280–#294): about 16 further API settings, the order precautions and their bypasses, smart-routing defaults, warning messages, and reqConfig/updateConfig.

### Fixed (6)

- #27 Answer reqConfig and updateConfig when configuration access is not available (10357)
- #274 Gateway setting: master API client ID
- #275 Gateway setting: download open orders on connection
- #276 Gateway setting: include FX positions when sending portfolio
- #277 Gateway setting: send zero positions for today's opening positions only
- #279 Gateway setting: use negative numbers to bind automatic orders

## Removed (present in ibx 0.7.1, gone now)

- `cancelOrderByPermId`, present in ibx 0.7.1: a request no gateway or reference client has.
- A TLS-verification bypass option, used only by the crate's own tests; a second tick-by-tick decoder family, used by the historical-tick path in ibx 0.7.1 and left unused once decoding moved to the streaming decoder.
- The automatic snap of order prices to the contract's tick grid (`snap_to_tick`, `OrderRequest::snap_prices`); `EClient::token_type()`, which always returned an empty string; `Order::has_extended_attrs()`; the unused Cargo feature `compat`.
- The `ibx` executable, the nine benchmark binaries and both bench harnesses, and several baseline examples; the Python test-injection methods (`_test_*`) from published wheels (test builds enable `test-helpers`); the release workflow that committed generated API pages, replaced by a drift check before release.
- A parallel account path, four commands nothing sent, an unused dependency, and redundant tests.

### Fixed (3)

- #106 A second, dead 35=E decoder family ships in the protocol surface, kept alive only by its tests
- #115 TLS-verification bypass is a public production-API option whose only true user is the crate's own tests
- #239 cancelOrderByPermId is a request nothing else has: no gateway wire type, no reference client

## Closed without a fix of their own: measured, documented or answered

- Questions answered, and reports settled by measurement or by comparison with a gateway: the venue itself sends nothing for a working market-on-close order in regular hours; on the measured paper session `ignore_size` changed nothing in the venue's answer; an option side with no price carries no greeks, as a gateway sends it; the two spellings of the historical-volatility series differ in a gateway too; the missing 3.13t wheel follows from the bindings dependency. One (#259) had already been delivered by the fix for #250, and two (#297, #298) by the fix for #275. The docs, a test or a workflow comment state each.

### Closed (10)

- #6 Market-on-close orders get no status while working
- #36 ignore_size has no effect on historical ticks
- #40 Tick ID for stock restrictions
- #44 Any plans for non-paper use?
- #56 Option ticks on a side with no price carry no greeks (the missing price reaches the model as NaN)
- #108 Wheel matrix omits 3.13t on a false pyo3 rationale; README publishes a wrong limit
- #166 The same venue series is refused under one spelling and accepted under the other: generic 512 is Never while its alias 104 is Always
- #259 An open order recovered after a reconnect states no multiplier or trading class
- #297 Initial order reports are delivered as live order and execution callbacks
- #298 Historical fills downloaded at connection trigger live callbacks

## Tests, CI, docs and tooling

- A full gate (`scripts/gate.py`): clippy on both feature sets, three library test configurations, doc tests, integration suites, rustdoc, the Python build and suite, paper-compat build, count and doc-generator checks.
- Tests: 4,818 offline and 182 that run against a paper session; behaviour fixes carry a regression test written to fail without the fix.
- CI: tests, docs deploy, nightly gateway-build drift check, paper-session workflow (dormant without secrets), release dry run.
- Documentation site: reference pages for requests and delivery, venue behaviour, limits, coverage (all 87 calls and 88 of 90 callbacks implemented; 78 request rows covered by the suites that run on a live session), wire coverage, beyond-the-API, recipes in Rust and Python, generated API references.
- Conformance and soak scripts, a paper harness, and wire captures used to check behaviour.

### Fixed (16)

- #32 Shutdown test depends on a one-second deadline under full parallel load
- #45 Symbol-match test can lose its request id to a parallel test
- #107 README guarantees its published counts cannot drift, but its own Testing table and example count are checked by nothing
- #110 Manifest and workflow comments state the dev-binary count as 30 and twenty-five; the crate holds 13
- #139 The nightly drift report tells readers the repository has issues turned off; issues are on
- #140 The docs-site pipeline downloads and runs an mdbook binary with no checksum, in a workflow holding pages:write and id-token:write
- #164 Nightly session workflow's live_python job builds without test-helpers and without ibapi, then runs the whole offline suite — guaranteed red on every scheduled run
- #199 scripts/conformance.py --compare spawns the capture binary without the dev-tools feature that binary requires, so the documented conformance check cannot run
- #200 The local gate compares only docs/ for generated-content drift while CI also compares README.md, so one stale README capabilities block passes the gate and then fails CI
- #236 The docs site builds without site-url, so the deployed 404 page links off the book and no sitemap is generated
- #238 session.yml hands venue credentials to jobs whose third-party actions are tag-pinned, not SHA-pinned
- #248 No test pins the completed time the open-order path renders, so a regression there passes the suite
- #255 docs.yml deploys with pages:write and id-token:write while every action it runs is pinned only by tag
- #258 The privileged-step pinning check misnames an abbreviated pin and accepts any comment beside it
- #265 No test pins the upper bound of the characters a connect host may carry
- #269 The scanner's refusal tests pin the refusal but not that nothing was sent, so a guard that reports and sends anyway survives

Also open at the time of writing: #156 (release setup on PyPI and crates.io) and #26 (advisor percentage allocation for attached orders, needs an advisor account); the gateway settings still to do are listed under Gateway settings.
