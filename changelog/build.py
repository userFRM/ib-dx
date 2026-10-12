import json,html,re,os
HERE=os.path.dirname(os.path.abspath(__file__))
iss=json.load(open(os.path.join(HERE,'issues.json')))
E=html.escape
SECTIONS=[
 ("identity","Identity, packaging and release"),
 ("session","Connection, login and session"),
 ("orders","Orders"),
 ("fills","Executions, commissions, positions and account"),
 ("mktdata","Market data"),
 ("options","Options"),
 ("hist","Historical data, news, fundamentals, scanners and calendar"),
 ("contracts","Contracts and reference data"),
 ("rust","Rust surface"),
 ("python","Python surface"),
 ("settings","Gateway settings"),
 ("removed","Removed (present in ibx 0.7.1, gone now)"),
 ("closed","Closed without a fix of their own: measured, documented or answered"),
 ("quality","Tests, CI, docs and tooling"),
]
FEAT={
"closed":["Questions answered, and reports settled by measurement or by comparison with a gateway: the venue itself sends nothing for a working market-on-close order in regular hours; on the measured paper session <code>ignore_size</code> changed nothing in the venue's answer; an option side with no price carries no greeks, as a gateway sends it; the two spellings of the historical-volatility series differ in a gateway too; the missing 3.13t wheel follows from the bindings dependency. One (#259) had already been delivered by the fix for #250. The docs, a test or a workflow comment state each."],
"identity":[
 "Renamed: ibx → ibkr-dx → <b>ib-dx</b>. Crate <code>ib-dx</code> (library <code>ib_dx</code>), Python package <code>ib-dx</code> (<code>import ib_dx</code>), environment variables <code>IB_DX_*</code>, docs at userfrm.github.io/ib-dx. An independent project, not an official Interactive Brokers product.",
 "Version reset to <b>0.1.0</b>, prepared for the first public release on crates.io and PyPI. Minimum Rust 1.89, Python 3.11+, free-threaded 3.14t supported.",
 "Release workflow: a <code>v*</code> tag builds the crate, the sdist and 25 wheels (Linux x86-64 and arm64, macOS arm64 and Intel, Windows; Python 3.11–3.14 and 3.14t). Once the registry setup in #156 is done it publishes to crates.io with a token held by the <code>release</code> environment and to PyPI through trusted publishing; both publishing jobs need the owner's approval, and a manual run publishes nothing.",
 "Crate builds only the library (no unused cdylib for Rust dependents, no <code>ibx</code> executable: its quote demo is <code>examples/quote_printer.rs</code>); repository tools build only with <code>--features dev-tools</code> and are left out of the published packages. PyPI metadata carries <code>License-Expression: AGPL-3.0-only</code>; the publishing jobs run only actions pinned by commit.",
 "The ib_async integration lives in its own repository, ib_async-dx, which runs ib_async on this engine; the README points there.",
],
"session":[
 "Announces the latest IB Gateway build at logon (now 10.51.1b), with the matching client string and launcher version; a nightly check fails when IB publishes a newer build.",
 "Name-service message version 52, with the session token's publish time on reconnect and on market-data logons.",
 "Second factor: IBKey code provider runs off the socket thread, the code is submitted as soon as it resolves and only while the socket is quiet; the authenticator-code second factor is supported; the second-factor timeout and cancel reach the caller.",
 "The server's Diffie-Hellman hello is validated instead of trusted; a frame failing signature verification is dropped; the known panic paths on gateway input are closed, and malformed-frame tests (truncated, mutated, invalid) guard them.",
 "Initial login tries the other built-in regional hosts when a default host does not answer. Reconnect tries every host this session was sent to, decides what to do from why the connection dropped, re-subscribes market data, and can be switched off (<code>reconnect_on_socket_err</code>).",
 "Connection loss is announced as a gateway does: 1100 when the trading connection goes, 1102 when it comes back, data-farm losses under their own notices; a program stays connected through a recoverable outage, and a data farm that cannot be recovered ends that feed without ending a healthy trading session.",
 "A placement written but not acknowledged when the connection drops is reconciled against the venue's working and completed orders after reconnect; one still unresolved is reported inactive, with 106 where a gateway states it; an unanswered cancel is re-sent while the order is still working.",
 "A session can be offered again after a restart: held in memory, written to disk only if asked. The server decides whether to take it; every restart observed so far has still asked for the ordinary login.",
 "Order ids persist per account and client id in a locked file keyed by a hash of the account, never the account number; next_valid_id is delivered at connect on both surfaces.",
 "A caller is told when another session already holds the account; a session can be opened that refuses to trade (read-only).",
 "Confirmed shutdown: disconnect says goodbye, drains, and reports how the engine ended.",
 "No part of the login name or session token appears in any log line.",
],
"orders":[
 "All 159 Order fields accounted for and documented: 128 sent, 23 taken and not sent exactly as a gateway sends nothing for them, 6 filled in by the venue on the way back, <code>transmit</code> acted on, and 1 not carried that says so on itself.",
 "Order types and attributes added or fixed on the wire: relative, pegged-to-benchmark with each reference field in its own tag, passive relative, peg-to-best, stop-protect, minute-based peg time in force, scale/ladder orders and hedges, discretion, position effect, combination legs with their own prices, conditional orders with only the flags each condition carries, conditionsIncludeOvernight. Some order types a gateway takes are not placed yet and are refused by name under 387 (volatility orders and the volatility pegs, LMT/REL + MKT and REL + LMT, trailing MIT/LIT, RPI, PEG STK and a few others).",
 "Attached orders: bracket families linked by parentId and transmit, preset attached orders constructed from the account's order presets, versioned client order ids, the originator tags on every leg, and wide order ids carried through. Group and model percentage allocation for preset children is still open (#26): state explicit quantities there.",
 "Cash-quantity orders checked, sized and sent as a gateway does, including fund orders, local refusals with a gateway's codes, and the size estimate shown on the order.",
 "Algorithm orders checked against the venue's own algorithm definitions, fetched once per session; algo-stage refusals 439/442/443/10205 with a gateway's texts.",
 "Replacing an order: brackets, algo and conditional orders, stops (the trigger moves) and the outside-RTH flag are carried; a modify the replace cannot express is refused instead of cancelling the order.",
 "Order status and openOrder published as a gateway publishes them: cumulative fill, held print, whyHeld (child, locate, trigger), PendingSubmit/ApiPending, unset prices as unset, the venue's message beside a status as error 399, time-in-force names decoded.",
 "Order callbacks follow the placing client, as a gateway's do (client 0 also hears orders placed outside the API); reqOpenOrders and reqCompletedOrders are scoped and dated as a gateway does.",
 "Global cancel waits for the venue's working orders to be named, cancels those and the ones placed since, drops held local placements, and says so when naming was incomplete. Cancels take <code>OrderCancel</code> (operator and manual-entry indicators); exercise carries manual time, customer account and professional-customer fields.",
 "Exercise and lapse an option position; what-if previews state the commission and the regular margin impact.",
 "Contracts named on every order path (index, bond, warrant, CFD, crypto, futures by maturity tag); orders refused for an unknown account, out-of-range or non-finite values, and quantities the wire cannot carry. Prices go out as stated: a price off the contract's tick grid is the venue's to refuse.",
],
"fills":[
 "Commission reports carry realized P&L, yield and redemption date, are told once per day key across reconnects, and follow their fill to its client.",
 "Executions recognised however many came before; a trade bust or correction is published as one signed execution, as a gateway does; execution and completed times follow the datetime-format setting.",
 "Execution queries include executions the venue restates at login, also after a process restart, without announcing them as new fills; <code>last_n_days</code> and <code>specific_dates</code> filters apply on the session timezone, with 320 for an invalid date.",
 "Positions and portfolio: daily P&L figures, holdings the account does not hold itself, positions on any account the login may act for, named portfolio contracts, and account values for each request that asked.",
 "Account figures delivered as the venue states them, with ledger figures kept apart from base-currency ones.",
],
"mktdata":[
 "Many of the venue's additional series read: the numbered figure tables kept whole, company figures and ratios, IPO estimate, average option volume, imbalance, the trade tape, odd lots, the venue's mark, halt and trading status, issuer details, short-sale restriction.",
 "Market data types 1–4 with the venue's frozen and delayed-frozen machinery: a closed market serves the frozen quote, delayed fallback for a login without live data, and the gateway's refusal codes (354, 10089) when a quote is refused.",
 "tickReqParams sent once, with the first data served; extra price precision and odd-lot quotes asked for where the logon offers them.",
 "Quote subscriptions held to the line allowance the login states (40 when none is stated), refused under 101 beyond it; callers sharing a contract share one line.",
 "More than one caller can watch the same contract; tick-by-tick (all last, bid/ask, midpoint) read record by record even when streams share a frame, carrying <code>number_of_ticks</code> and <code>ignore_size</code>; real-time bars with several bars per frame; depth with smart and regular books held apart, the requested row window kept as levels move, and a book reset before the rebuilt levels after a reconnect.",
 "Request validation as a gateway: duplicate ids (322 with a gateway's text), empty exchange, BAG without legs, unknown generic ticks, snapshot rate, currencies outside the session's set, unknown market data type; generic tick 787 refused under 10367 where the session was not offered odd lots. The fundamentals series (258) is forwarded: a gateway's feature gate on it is documented as not reproduced.",
 "Option snapshots collect the computations available and end after eleven seconds; regulatory snapshots are decoded, with the permission check, timeout and missing-exchange-map error documented as differing from a gateway.",
],
"options":[
 "The venue's own option model read, with bid/ask/last/model computations (10–13, 80–83) built from the venue's model data, currency rate curves, dividend schedules with their ex-dates, and the underlying's own quote and mark. Documented differences remain: frozen model ticks use the live model, and VIX-style, quanto, forex-index and futures-style premium cases.",
 "Standalone option prices and greeks; implied volatility and option price calculations from the venue's model; option chains per trading class.",
],
"hist":[
 "Historical bars: every series the venue carries for stocks and funds asked along the contract's id history, ADJUSTED_LAST adjusted for dividends, keepUpToDate continuing from the last historical bar (a daily bar keeps the history's session end, then rolls at midnight UTC), crypto aggregated trades mapped as a gateway maps them, a request that can be withdrawn while still being served, at most 50 queries in flight.",
 "Validation as a gateway: bar sizes, durations (seven refusals), date formats, limited history (166), whatToShow per request type, endDateTime parsing (10314), duplicate ids, head timestamp and histogram rules.",
 "Historical ticks (start and end), head timestamps, histograms, corporate actions read rather than discarded.",
 "News: providers, historical headlines with their window, articles, bulletins kept across calls; fundamentals forwarded as a gateway forwards them.",
 "Scanners: subscription filters carried to the wire, permission notices (490/492), denied codes refused (10359/10360), concurrency capped by the logon's message rate, market name on each row.",
 "Corporate-events calendar: metadata, event data, two request slots and the gateway's 10278–10309 refusals; the venue's clock answerable in milliseconds.",
],
"contracts":[
 "Contracts looked up by id alone for every security type but BAG (refused under 321, as a gateway does), by description when no id is given, and refused before sending as a gateway refuses them (identifier, security type, non-ASCII symbol, expiry and strike, zero strike, unset id marker).",
 "Every field and every identifier a definition states is kept (CUSIP and others), with market rules (price and size tables), bond terms, fund charges, the economic-value rule, the underlying and the time trading stops. Trading and liquid hours are rendered on the timezone named beside them.",
 "A large lookup times out on silence rather than on total time, and a timeout states how many definitions arrived.",
 "Display groups served; the Nasdaq listing named as a gateway names it (ISLAND when granted and enabled).",
],
"rust":[
 "A TWS API shaped EClient/EWrapper: all 87 calls of the canonical list and 88 of its 90 callbacks (the two others declared as stubs), with Wrapper::error carrying its error time and req_ids taking its count.",
 "Requests and cancels return nothing and take no wrapper: they return once admitted, and every answer, refusal, connection notice and order report arrives in session order through <code>process_msgs</code>. <code>ErrorOrigin</code> and <code>error_from</code> say whether an error belongs to a request, an order, the session or the client, beside the ordinary <code>error</code>.",
 "Sizes, positions and volumes delivered as a Decimal with the reference's unset marker; quote reads return a QuoteSnapshot with prices as numbers; <code>account()</code> returns <code>None</code> until the account download is complete.",
 "Calls that answer (contracts, bars, option chains, holdings, account values, scans, schedules), optional events on a bounded standard-library channel, <code>wait_for_data()</code> and <code>on_data()</code> to wake a caller's event loop, and <code>traffic()</code> for the session's byte and message totals.",
],
"python":[
 "The reference client's module layout, names and spellings, so a callback-based ibapi program moves with an import change (TWSSyncWrapper is not provided): module aliases, constructors taking the reference spelling, keyword arguments, enum and printing helpers, record classes printing as ibapi prints them, order conditions with their text.",
 "Callbacks delivered in the shape the program's wrapper declares (3, 4 or 5 argument error; commissionReport or commissionAndFeesReport).",
 "Sizes delivered as Decimal where the reference delivers Decimal; unset values kept unset; None handled as the reference handles it at send time.",
 "Calls that answer on the Python surface too, and a caller's own refusal can be put into the ordered stream.",
 "The reference client's 10.50 protobuf request family (the <code>…ProtoBuf</code> calls) routed to the text requests, with the exceptions documented (configuration calls answer 10357, <code>useProtoBuf</code> is False); <code>setOptionalCapabilities</code> and the client-side <code>validate*</code> helpers a moved program calls.",
 "Blocking login, waits and shutdown release the GIL; KeyboardInterrupt and SystemExit raised in a callback reach the caller; clients take part in cyclic garbage collection.",
],
"settings":[
 "Gateway settings on the client: read-only API, timezone and locale, date-time format for timestamps, which execution reports are downloaded, the older Nasdaq spelling, reconnect on socket error, the announced build, the master API client id (<code>master_client_id</code>), and the announced network identity (the connection's own interface, or <code>mac_address</code>/<code>IB_DX_MAC</code> and <code>lan_ip</code>/<code>IB_DX_IP</code>). <code>setServerLogLevel</code> changes this client's logger.",
 "Still to do, tracked in #273 (#275–#294): about 20 further API settings, the order precautions and their bypasses, smart-routing defaults, warning messages, and reqConfig/updateConfig.",
],
"removed":[
 "<code>cancelOrderByPermId</code>, present in ibx 0.7.1: a request no gateway or reference client has.",
 "A TLS-verification bypass option, used only by the crate's own tests; a second tick-by-tick decoder family, used by the historical-tick path in ibx 0.7.1 and left unused once decoding moved to the streaming decoder.",
 "The automatic snap of order prices to the contract's tick grid (<code>snap_to_tick</code>, <code>OrderRequest::snap_prices</code>); <code>EClient::token_type()</code>, which always returned an empty string; <code>Order::has_extended_attrs()</code>; the unused Cargo feature <code>compat</code>.",
 "The <code>ibx</code> executable, the nine benchmark binaries and both bench harnesses, and several baseline examples; the Python test-injection methods (<code>_test_*</code>) from published wheels (test builds enable <code>test-helpers</code>); the release workflow that committed generated API pages, replaced by a drift check before release.",
 "A parallel account path, four commands nothing sent, an unused dependency, and redundant tests.",
],
"quality":[
 "A full gate (<code>scripts/gate.py</code>): clippy on both feature sets, three library test configurations, doc tests, integration suites, rustdoc, the Python build and suite, paper-compat build, count and doc-generator checks.",
 "Tests: 4,796 offline and 182 that run against a paper session; behaviour fixes carry a regression test written to fail without the fix.",
 "CI: tests, docs deploy, nightly gateway-build drift check, paper-session workflow (dormant without secrets), release dry run.",
 "Documentation site: reference pages for requests and delivery, venue behaviour, limits, coverage (all 87 calls and 88 of 90 callbacks implemented; 78 request rows covered by the suites that run on a live session), wire coverage, beyond-the-API, recipes in Rust and Python, generated API references.",
 "Conformance and soak scripts, a paper harness, and wire captures used to check behaviour.",
]}
RULES=[
 ("python",r"python|ibapi|reference client'?s? (module|class|helper)|EWrapper base|TickTypeEnum|getEnumTypeName|dir\(\)|Enum class|notebook|floatMaxString|repr|OrderCondition|condition classes|PriceCondition|data classes|keyword|BarData carries|valueToString"),
 ("rust",r"Rust|native"),
 ("identity",r"release|wheel|README|Manifest|docs-site|docs site|session\.yml|docs\.yml|pinning|nightly|build tracker|drift|counts"),
 ("options",r"[Oo]ption (comput|ticks|chains)|greeks|Frozen market data: option"),
 ("hist",r"[Hh]istor|head ?[Tt]ime|histogram|news|[Ff]undamental|scanner|[Ss]canner|calendar|event data|metadata request|bar query|keepUpToDate|bar-size|bars? |formatDate|duration|YIELD_BID_ASK|matching symbols|reqMatchingSymbols|corporate"),
 ("fills",r"[Ee]xecution|commission|[Pp]osition|portfolio|AccruedCash|AccountCode|account|fill|trade bust|completedTime"),
 ("orders",r"[Oo]rder|[Bb]racket|cancel|[Cc]ash-quantity|[Cc]ash quantity|algo|whyHeld|what-if|margin condition|permId|lmtPrice|time-in-force|ApiPending|[Cc]onditional"),
 ("mktdata",r"[Mm]arket ?data|reqMktData|tick|[Dd]epth|real-time|generic|snapshot|quote|tickReqParams|creditman|NEWS"),
 ("contracts",r"[Cc]ontract|identifier|strike|lastTradeDate|evMultiplier|secIdList|minAlgoSize|market rule|reqMarketRule"),
 ("session",r"[Cc]onnect|login|logon|[Ss]hutdown|clock|log directory|feature tokens|session"),
 ("quality",r"test|[Tt]est |gate|conformance|CI"),
]
groups={k:[] for k,_ in SECTIONS}
OVR={274: 'settings', 27: 'settings', 30: 'orders', 64: 'orders', 75: 'orders', 83: 'mktdata', 85: 'mktdata', 87: 'mktdata', 95: 'hist', 97: 'hist', 106: 'removed', 114: 'orders', 115: 'removed', 120: 'mktdata', 127: 'python', 132: 'python', 154: 'hist', 161: 'hist', 169: 'python', 188: 'hist', 193: 'python', 209: 'hist', 228: 'hist', 253: 'fills', 271: 'orders', 165: 'orders', 197: 'orders', 242: 'session', 32: 'quality', 1: 'orders', 3: 'orders', 39: 'orders', 72: 'orders', 91: 'orders', 152: 'orders', 179: 'orders', 105: 'python', 155: 'python', 21: 'python', 41: 'python', 128: 'python', 131: 'python', 125: 'hist', 205: 'mktdata', 23: 'mktdata', 68: 'mktdata', 206: 'mktdata', 246: 'mktdata', 235: 'session', 107: 'quality', 110: 'quality', 139: 'quality', 200: 'quality', 236: 'quality', 265: 'quality', 248: 'quality', 269: 'quality', 164: 'quality', 239: 'removed', 244: 'rust', 140: 'quality', 238: 'quality', 255: 'quality', 258: 'quality', 121: 'mktdata', 159: 'mktdata', 203: 'mktdata', 226: 'contracts', 227: 'contracts', 256: 'contracts', 262: 'contracts', 185: 'hist'}
NOTE={89: " (787 now refused; the 258 gate is documented as not reproduced)"}
for i in sorted(iss,key=lambda x:x['number']):
    t=i['title']; n=i['number']
    if n in (26,156,273): continue
    if n in (6,36,40,44,56,108,166,259):
        groups['closed'].append((n,t)); continue
    sec=OVR.get(n)
    if sec:
        groups[sec].append((n,t)); continue
    for k,pat in RULES:
        if re.search(pat,t): sec=k; break
    groups[sec or 'quality'].append((n,t))
total_fixed=sum(len(v) for k,v in groups.items() if k!='closed')
out=[]
for k,name in SECTIONS:
    feats=FEAT.get(k,[]); fixes=groups[k]
    out.append(f'<section id="{k}" class="area"><h2>{E(name)}</h2>')
    if feats:
        out.append('<h3>What it does now</h3><ul class="feat">'+''.join(f'<li>{f}</li>' for f in feats)+'</ul>')
    if fixes:
        out.append(f'<h3>{'Closed' if k=='closed' else 'Fixed'} ({len(fixes)})</h3><ul class="fix">'+''.join(f'<li><a href="https://github.com/userFRM/ib-dx/issues/{n}">#{n}</a> {E(t)}{E(NOTE.get(n,""))}</li>' for n,t in fixes)+'</ul>')
    out.append('</section>')
toc=''.join(f'<li><a href="#{k}">{E(n)}</a> <span class="n">{len(groups[k])}</span></li>' for k,n in SECTIONS)
page=open(os.path.join(HERE,'template.html')).read().replace('%%TOC%%',toc).replace('%%BODY%%','\n'.join(out)).replace('%%FIXED%%',str(total_fixed))
open(os.path.join(HERE,'net-changelog.html'),'w').write(page)
print({k:len(v) for k,v in groups.items()}, total_fixed)

# Markdown twin of the page, for the repository.
import html as _h
def MD(s):
    s = re.sub(r'</?b>', '**', s)
    s = re.sub(r'</?code>', '`', s)
    return _h.unescape(re.sub(r'<[^>]+>', '', s))
md = ['# ib-dx: everything since ibx 0.7.1', '',
      "One netted changelog, as if the work since DeepEntropy's last release (ibx 0.7.1, 25 July 2026) were a single squash merge. Each line states the end result on `main` (0e659eac, 12 October 2026); work that was redone later appears once, in its final form. Fixes are listed by the issue that tracked them.", '',
      f'- ibx 0.7.1 → ib-dx 0.1.0', f'- 1,970 commits, 557 files changed, 44k → 218k lines of Rust in `src/`',
      f'- {total_fixed} issues fixed on ib-dx, {len(groups["closed"])} closed without a fix of their own; 146 fork-era issue and PR references',
      f'- 4,796 offline tests + 182 that run against a paper session', '']
for k, name in SECTIONS:
    md.append(f'## {name}'); md.append('')
    for f in FEAT.get(k, []): md.append(f'- {MD(f)}')
    if FEAT.get(k): md.append('')
    if groups[k]:
        md.append(f"### {'Closed' if k=='closed' else 'Fixed'} ({len(groups[k])})"); md.append('')
        for n, t in groups[k]: md.append(f'- #{n} {t}{NOTE.get(n, "")}')
        md.append('')
md.append('Also open at the time of writing: #156 (release setup on PyPI and crates.io), #26 (advisor percentage allocation for attached orders, needs an advisor account) and #296 (openOrder for an order the session is told of but did not place); the gateway settings still to do are listed under Gateway settings.')
open(os.path.join(HERE,'..','CHANGELOG.md'),'w').write('\n'.join(md) + '\n')
