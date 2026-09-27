//! Historical data queries via the data connection.
//!
//! Responses contain XML ResultSetBar with OHLCV bar data.

use crate::control::xml::tag;
use crate::protocol::fix;

// Tags for historical data
/// FIX tag 6118: the historical xml.
pub const TAG_HISTORICAL_XML: u32 = 6118;

/// Whether a bar request asks for the split- and dividend-adjusted series.
///
/// The venue carries no adjusted series to pass through: what it serves is raw
/// trades, and an adjusted series is those trades folded with the contract's
/// own corporate actions. So `ADJUSTED_LAST` is not a wire type — it is not in
/// [`BarDataType::from_api_str`], which knows only what the venue streams — and
/// the historical paths read it here to fetch raw trades and fold them rather
/// than send a name the venue would answer "no historical market data" to. Read
/// before the wire table, case folded, so the reference client's own spelling
/// is taken whatever the casing.
pub fn what_to_show_is_adjusted(what_to_show: &str) -> bool {
    what_to_show.eq_ignore_ascii_case("ADJUSTED_LAST")
}

/// Bar data types for historical queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarDataType {
    /// What traded.
    Trades,
    /// The midpoint between bid and ask.
    Midpoint,
    /// The bid.
    Bid,
    /// The ask.
    Ask,
    /// Both sides.
    BidAsk,
    /// Trades, aggregated as the venue aggregates them.
    AggTrades,
    /// The rate charged to borrow the stock.
    FeeRate,
    /// The yield at the bid.
    YieldBid,
    /// The yield at the ask.
    YieldAsk,
    /// The yield at either side, asked as the pair it is.
    ///
    /// The venue carries the two sides as two series and neither holds both,
    /// so a gateway asks the bid-yield series with the ask-yield series added
    /// to the query, and answers the pair from both — see [`BarDataType::paired_series`].
    YieldBidAsk,
    /// The yield at the last trade.
    YieldLast,
    /// The yield at the venue's own mark.
    YieldMark,
    /// A fund's net asset value.
    NavLast,
    /// The volatility the underlying realised.
    HistoricalVolatility,
    /// The volatility its options implied.
    ImpliedVolatility,
    /// The price and size an auction is indicating it would match at.
    IndicativeAuctionPriceSize,
    /// Open interest across the calls written on this contract.
    CallOptionOpenInterest,
    /// Open interest across the puts written on it.
    PutOptionOpenInterest,
    /// Volume across the calls written on it.
    CallOptionVolume,
    /// Volume across the puts written on it.
    PutOptionVolume,
}

impl BarDataType {
    /// Read the official API's `what_to_show` string.
    ///
    /// An empty string is the documented TRADES default; anything else must
    /// match exactly, case aside. A value this does not know is an error
    /// rather than a fallback — a misspelled "BID" answered as trade bars
    /// looks like data.
    pub fn from_api_str(s: &str) -> Result<BarDataType, String> {
        Ok(match s.to_uppercase().as_str() {
            "" | "TRADES" => Self::Trades,
            "MIDPOINT" => Self::Midpoint,
            "BID" => Self::Bid,
            "ASK" => Self::Ask,
            "BID_ASK" => Self::BidAsk,
            // Not a name the venue answers to, and not a bar it streams: an
            // adjusted series is built from the raw trades and the contract's
            // own actions, not served ready-made. The bar requests fold the
            // two — see [`what_to_show_is_adjusted`], which the historical
            // paths read before this table — so a real-time bar, which has no
            // history to fold and no actions in hand, is refused rather than
            // answered with raw trades under an adjusted name.
            "ADJUSTED_LAST" => return Err(
                "an adjusted series is built from the raw trades and the contract's own \
                 actions, so it is not a live bar the venue streams. Ask for it on a \
                 historical bar request — `req_historical_data` and \
                 `EClient::historical_data` both serve it. Do not fold TRADES a second \
                 time: it already comes back adjusted for splits"
                    .to_string(),
            ),
            "AGGTRADES" => Self::AggTrades,
            "FEE_RATE" => Self::FeeRate,
            "YIELD_BID" => Self::YieldBid,
            "YIELD_ASK" => Self::YieldAsk,
            // Two series the venue carries separately and neither holds both.
            // A gateway asks it as the bid-yield series with the ask-yield
            // one added to the query, and answers the pair from both.
            "YIELD_BID_ASK" => Self::YieldBidAsk,
            "YIELD_LAST" => Self::YieldLast,
            "YIELD_MARK" => Self::YieldMark,
            "NAV_LAST" => Self::NavLast,
            "HISTORICAL_VOLATILITY" => Self::HistoricalVolatility,
            "OPTION_IMPLIED_VOLATILITY" => Self::ImpliedVolatility,
            "INDICATIVE_AUCTION_PRICE_SIZE" => Self::IndicativeAuctionPriceSize,
            // Open interest and volume, one series per side of the chain and
            // stated against the contract the options are written on rather
            // than against an option. Historical open interest is not served
            // any other way, so refused here it could not be asked for at all.
            "CALL_OPTION_OPEN_INTEREST" => Self::CallOptionOpenInterest,
            "PUT_OPTION_OPEN_INTEREST" => Self::PutOptionOpenInterest,
            "CALL_OPTION_VOLUME" => Self::CallOptionVolume,
            "PUT_OPTION_VOLUME" => Self::PutOptionVolume,
            other => {
                return Err(format!(
                    "Unsupported what_to_show '{other}': expected TRADES, MIDPOINT, \
                     BID, ASK, BID_ASK, AGGTRADES, FEE_RATE, YIELD_BID, YIELD_ASK, \
                     YIELD_BID_ASK, YIELD_LAST, YIELD_MARK, NAV_LAST, HISTORICAL_VOLATILITY, \
                     OPTION_IMPLIED_VOLATILITY, INDICATIVE_AUCTION_PRICE_SIZE, \
                     CALL_OPTION_OPEN_INTEREST, PUT_OPTION_OPEN_INTEREST, \
                     CALL_OPTION_VOLUME or PUT_OPTION_VOLUME",
                ));
            }
        })
    }

    /// Whether a gateway puts a series of this kind on the scale of the
    /// actions that move it: the ones priced as the contract trades — what
    /// traded, the midpoint, either side or both, a fund's net asset value and
    /// an auction's indicated price. A volatility, a yield or a rate it leaves
    /// as the venue served it.
    pub(crate) fn is_adjusted(&self) -> bool {
        matches!(
            self,
            Self::Trades | Self::Midpoint | Self::Bid | Self::Ask | Self::BidAsk | Self::AggTrades
                | Self::NavLast | Self::IndicativeAuctionPriceSize,
        )
    }

    /// The name the venue knows this by.
    ///
    /// Not the name the reference client uses, and not always the obvious
    /// casing: the midpoint is `MidPoint`, and sent as `Midpoint` the venue
    /// answers "no historical market data" — which reads as a series that does
    /// not exist rather than a name it does not know. Asked and answered
    /// against a session; see `probe_midpoint`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trades => "Last",
            Self::Midpoint => "MidPoint",
            Self::Bid => "Bid",
            Self::Ask => "Ask",
            Self::BidAsk => "BidAsk",
            Self::AggTrades => "AggLast",
            Self::FeeRate => "FeeRate",
            Self::YieldBid => "BidYield",
            Self::YieldAsk => "AskYield",
            // The pair is asked under its bid side, with the ask side added
            // to the query beside it — see [`BarDataType::paired_series`].
            Self::YieldBidAsk => "BidYield",
            Self::YieldLast => "LastYield",
            Self::YieldMark => "MarkYield",
            Self::NavLast => "NavLast",
            Self::HistoricalVolatility => "HistVol",
            Self::ImpliedVolatility => "OptionImpliedVol",
            Self::IndicativeAuctionPriceSize => "AuctionIndicLast",
            Self::CallOptionOpenInterest => "CallOpenInterest",
            Self::PutOptionOpenInterest => "PutOpenInterest",
            // The venue names the option-volume series after the last trade
            // and not after the volume. Read as a typo and "corrected" to
            // something with Volume in it, both come back refused.
            Self::CallOptionVolume => "CallLast",
            Self::PutOptionVolume => "PutLast",
        }
    }

    /// The second series a gateway adds to the query for a name that is a
    /// pair, and `None` for every name that is one series.
    ///
    /// The venue carries the two sides separately and neither holds both, so
    /// the query goes out as the bid side under [`BarDataType::as_str`] with
    /// the ask side beside it under this name, each under an id of its own,
    /// and the answer is the pair folded from both — see [`merge_pair`].
    pub fn paired_series(&self) -> Option<&'static str> {
        match self {
            Self::YieldBidAsk => Some("AskYield"),
            _ => None,
        }
    }
}

/// Bar size / time step for historical queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarSize {
    /// One bar covers a second.
    Sec1,
    /// One bar covers five seconds.
    Sec5,
    /// One bar covers ten seconds.
    Sec10,
    /// One bar covers fifteen seconds.
    Sec15,
    /// One bar covers thirty seconds.
    Sec30,
    /// One bar covers a minute.
    Min1,
    /// One bar covers two minutes.
    Min2,
    /// One bar covers three minutes.
    Min3,
    /// One bar covers four minutes.
    Min4,
    /// One bar covers five minutes.
    Min5,
    /// One bar covers ten minutes.
    Min10,
    /// One bar covers fifteen minutes.
    Min15,
    /// One bar covers twenty minutes.
    Min20,
    /// One bar covers half an hour.
    Min30,
    /// One bar covers an hour.
    Hour1,
    /// One bar covers two hours.
    Hour2,
    /// One bar covers three hours.
    Hour3,
    /// One bar covers four hours.
    Hour4,
    /// One bar covers eight hours.
    Hour8,
    /// One bar covers a day.
    Day1,
    /// One bar covers a week.
    Week1,
    /// One bar covers a month.
    Month1,
    /// One bar covers a quarter.
    Month3,
    /// One bar covers a year.
    Year1,
}

/// The legal bar-size names and the sizes they name, in the order a gateway
/// lists them. The last two are legal and matched, but are left out of the
/// list a gateway renders in its refusal: it names every size but those two.
const BAR_SIZES: [(&str, BarSize); 24] = [
    ("1 secs", BarSize::Sec1),
    ("5 secs", BarSize::Sec5),
    ("10 secs", BarSize::Sec10),
    ("15 secs", BarSize::Sec15),
    ("30 secs", BarSize::Sec30),
    ("1 min", BarSize::Min1),
    ("2 mins", BarSize::Min2),
    ("3 mins", BarSize::Min3),
    ("4 mins", BarSize::Min4),
    ("5 mins", BarSize::Min5),
    ("10 mins", BarSize::Min10),
    ("15 mins", BarSize::Min15),
    ("20 mins", BarSize::Min20),
    ("30 mins", BarSize::Min30),
    ("1 hour", BarSize::Hour1),
    ("2 hours", BarSize::Hour2),
    ("3 hours", BarSize::Hour3),
    ("4 hours", BarSize::Hour4),
    ("8 hours", BarSize::Hour8),
    ("1 day", BarSize::Day1),
    ("1W", BarSize::Week1),
    ("1M", BarSize::Month1),
    ("3 months", BarSize::Month3),
    ("1 year", BarSize::Year1),
];

impl BarSize {
    /// Read the official API's bar-size string.
    ///
    /// The one table every request path reads, holding the whole legal set of
    /// names, matched exactly and case-sensitively: a gateway compares the
    /// size against each legal name by strict equality and refuses a miss
    /// before asking the venue. An alias or a casing the venue would happen
    /// to answer got bars where a gateway gave an error — plausible,
    /// complete candles the caller could not have asked a real gateway for.
    /// A miss is refused in the words a gateway uses, naming the legal ones
    /// the way it names them: the list it renders leaves out the two longest
    /// sizes, which are legal all the same.
    pub fn from_api_str(s: &str) -> Result<BarSize, String> {
        BAR_SIZES
            .iter()
            .find(|(name, _)| *name == s)
            .map(|(_, size)| *size)
            .ok_or_else(|| {
                format!(
                    "Historical data bar size setting is invalid. Legal ones are: {}",
                    BAR_SIZES[..BAR_SIZES.len() - 2]
                        .iter()
                        .map(|(name, _)| *name)
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            })
    }

    /// Whether a bar this long can be kept up to date.
    ///
    /// What the venue keeps sending after the batch is five-second bars, and
    /// the bar still forming is folded from those. So a size that is a whole
    /// number of them can be formed and one that is not cannot: a second is
    /// shorter than what arrives, and folding into it relabelled each
    /// five-second bar as a one-second one and handed the caller five times
    /// the volume under a size it never traded in.
    ///
    /// A week and a month are formed on the calendar rather than on a multiple
    /// of their length: a week opens on its Monday and a month on its first
    /// day, both at midnight UTC, which is where a gateway folds them. The
    /// venue states the bounds of the bars it aggregated — `date` and
    /// `endDate` — and they run from a week's first trading day to the
    /// Saturday after it, and from a month's to the first of the next. A day
    /// opens at midnight, which is a boundary, and is documented as the one
    /// it is.
    ///
    /// This is what this client can form, not what the venue accepts — nothing
    /// on the wire says a size may not be kept up to date. The list it replaces
    /// named five sizes, refusing sixteen that fold exactly and admitting the
    /// one that cannot.
    pub fn supports_keep_up_to_date(&self) -> bool {
        let seconds = self.seconds();
        matches!(self, Self::Week1 | Self::Month1)
            || (seconds >= 5 && seconds.is_multiple_of(5) && seconds <= 86_400)
    }

    /// How long one of these lasts.
    ///
    /// What a bar covers, so a bar still forming can be folded from the
    /// five-second bars the venue streams.
    pub fn seconds(&self) -> u32 {
        match self {
            Self::Sec1 => 1,
            Self::Sec5 => 5,
            Self::Sec10 => 10,
            Self::Sec15 => 15,
            Self::Sec30 => 30,
            Self::Min1 => 60,
            Self::Min2 => 120,
            Self::Min3 => 180,
            Self::Min4 => 240,
            Self::Min5 => 300,
            Self::Min10 => 600,
            Self::Min15 => 900,
            Self::Min20 => 1_200,
            Self::Min30 => 1_800,
            Self::Hour1 => 3_600,
            Self::Hour2 => 7_200,
            Self::Hour3 => 10_800,
            Self::Hour4 => 14_400,
            Self::Hour8 => 28_800,
            Self::Day1 => 86_400,
            Self::Week1 => 604_800,
            Self::Month1 => 2_592_000,
            // A quarter is three of the gateway's 31-day months and a year
            // is its 365 days, which is how it counts both.
            Self::Month3 => 8_035_200,
            Self::Year1 => 31_536_000,
        }
    }

    /// The name the venue knows this by.
    ///
    /// A week and a month are the two that are not their API spelling. Sent as
    /// `1 week` the venue answers six months with a hundred and twenty-four
    /// bars — one a day — and sent as `1 month` it answers with one a minute;
    /// sent as `1W` and `1M` it answers twenty-six and six, and restates the
    /// step it used in the reply. So the API spelling was being answered at a
    /// size nobody asked for and handed on under the size they did. Everything
    /// shorter goes out under the name a caller wrote.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sec1 => "1 secs",
            Self::Sec5 => "5 secs",
            Self::Sec10 => "10 secs",
            Self::Sec15 => "15 secs",
            Self::Sec30 => "30 secs",
            Self::Min1 => "1 min",
            Self::Min2 => "2 mins",
            Self::Min3 => "3 mins",
            Self::Min4 => "4 mins",
            Self::Min5 => "5 mins",
            Self::Min10 => "10 mins",
            Self::Min15 => "15 mins",
            Self::Min20 => "20 mins",
            Self::Min30 => "30 mins",
            Self::Hour1 => "1 hour",
            Self::Hour2 => "2 hours",
            Self::Hour3 => "3 hours",
            Self::Hour4 => "4 hours",
            Self::Hour8 => "8 hours",
            Self::Day1 => "1 day",
            Self::Week1 => "1W",
            Self::Month1 => "1M",
            Self::Month3 => "3 months",
            Self::Year1 => "1 year",
        }
    }
}

/// Parameters for a historical data request.
#[derive(Debug, Clone)]
pub struct HistoricalRequest {
    /// The name this client gave the query, which the answer echoes.
    pub query_id: String,
    /// The venue's id for the contract.
    pub con_id: u32,
    /// Its ticker.
    pub symbol: String,
    /// Wire security type and exchange for the contract being requested.
    /// Owned rather than static: they come from the caller's `Contract`, and
    /// hardcoding them described a different contract than was asked for.
    pub sec_type: String,
    /// Which venue to answer for.
    pub exchange: String,
    /// Which series is wanted.
    pub data_type: BarDataType,
    /// The end of the window. Empty means now.
    pub end_time: String,
    /// How far back from that end it reaches.
    pub duration: String,
    /// How long one bar covers.
    pub bar_size: BarSize,
    /// Whether to count only regular trading hours.
    pub use_rth: bool,
    /// Whether the venue keeps sending once the window is answered.
    pub keep_up_to_date: bool,
    /// Whether a contract that has already expired is in scope.
    ///
    /// Stated on every query, and stated as the caller set it. Written as a
    /// flat `no`, a request for a settled future asked about a contract that
    /// no longer exists and came back empty.
    pub include_expired: bool,
}

/// A single historical OHLCV bar parsed from XML.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoricalBar {
    /// When the bar opened.
    pub time: String,
    /// Its first price.
    pub open: f64,
    /// Its highest.
    pub high: f64,
    /// Its lowest.
    pub low: f64,
    /// Its last.
    pub close: f64,
    /// How much traded in it.
    pub volume: i64,
    /// The volume-weighted average price.
    pub wap: f64,
    /// How many trades made it.
    ///
    /// Signed, and the width every surface reports it in. Held wider, a count
    /// past what that width carries reached a caller negative — a bar made by
    /// minus two billion trades. A stated count that will not fit is not one
    /// this client can carry, and is read as a bar that states none.
    pub count: i32,
    /// When the bar closed, as the venue states it.
    ///
    /// A bar the venue aggregated states its own bounds, and those bounds are
    /// not derivable from the start: a week is stated from its first trading
    /// day to the Saturday after it, and a month to the first of the next.
    ///
    /// The last bar of any series is normally partial. Stated, this says where
    /// the bar actually ends, so a caller can tell a finished week from a
    /// running one rather than inferring it from the calendar and the clock.
    ///
    /// Empty where the venue stated none. It states one under `endTime` on
    /// the sizes it times and under `endDate` on the two it dates, and this is
    /// read under both.
    pub end: String,
}

/// Parsed historical data response.
#[derive(Debug, Clone)]
pub struct HistoricalResponse {
    /// The name this client gave the query, which the answer echoes.
    pub query_id: String,
    /// The zone the times are stated in.
    pub timezone: String,
    /// The bars themselves.
    pub bars: Vec<HistoricalBar>,
    /// Whether this is the last part of the answer.
    pub is_complete: bool,
}

/// The duration a query carries, spelled the way a gateway spells it.
///
/// A bare number is a number of seconds, and a gateway appends the unit
/// before it sends: asked unit-less, the venue was handed a length it read
/// as it pleased. The unit's case is then folded to the spelling the venue
/// takes — seconds and weeks uppercase, days, months and years lowercase —
/// which is not one case or the other: the wrong case is refused outright
/// with "Invalid time length" rather than corrected. Measured against a live
/// session, every unit, both cases.
///
/// Anything else passes through as it stands. On the bar path nothing else
/// reaches here — [`validate_duration`] refuses it where the request is
/// taken — and where a path of its own sends a duration this does not
/// validate, refusing it here would hide the venue's answer about what it
/// accepts.
pub fn normalize_duration(duration: &str) -> String {
    fold_duration(duration.trim())
}

/// What a gateway does to the duration text before it reads it: a bare
/// number becomes that many seconds, and each unit is folded to the case
/// the venue takes. No trimming — the gateway reads the text as it stands.
fn fold_duration(duration: &str) -> String {
    if !duration.is_empty() && duration.bytes().all(|b| b.is_ascii_digit()) {
        return format!("{duration} S");
    }
    duration
        .chars()
        .map(|c| match c {
            's' => 'S',
            'D' => 'd',
            'w' => 'W',
            'M' => 'm',
            'Y' => 'y',
            other => other,
        })
        .collect()
}

/// What a request's duration has to state to be asked, refused in a gateway's
/// own words where it does not.
///
/// Taken where the request is taken, before the venue is asked: an empty
/// duration, one that is not an integer, a space and a unit of seconds, days,
/// weeks, months or years, and one whose count is outside the range a gateway
/// enforces on its unit — seconds from thirty to a day, a year of days, two
/// score of weeks, twelve months — are each refused there, and a bare number
/// is read as the seconds it is. A unit outside the five is a miss of the
/// format rather than a length the venue is asked about: a minute, an hour
/// and a quarter were once folded into the query and answered as whatever
/// the venue made of them.
pub fn validate_duration(duration: &str) -> Result<(), String> {
    const FORMAT: &str = "When specifying a unit, historical data request duration format is \
                          integer{SPACE}unit (S|D|W|M|Y).";
    if duration.is_empty() {
        return Err("Historical data request duration not specified.".to_string());
    }
    let folded = fold_duration(duration);
    // An integer, one space and one unit — the shape a gateway matches the
    // folded duration against, whole.
    let bytes = folded.as_bytes();
    let shaped = bytes.len() >= 3
        && "dSWmy".contains(bytes[bytes.len() - 1] as char)
        && bytes[bytes.len() - 2] == b' '
        && bytes[..bytes.len() - 2].iter().all(|b| b.is_ascii_digit());
    if !shaped {
        return Err(FORMAT.to_string());
    }
    // A count past what the width a gateway reads it in carries is one it
    // refuses rather than reads around.
    let Ok(count) = folded[..folded.len() - 2].parse::<i32>() else {
        return Err("Historical data requested duration is invalid.".to_string());
    };
    let unit = bytes[bytes.len() - 1] as char;
    if count < 1 || (unit == 'S' && count < 30) {
        return Err("Historical data requested duration is invalid.".to_string());
    }
    let why = match (unit, count) {
        ('S', count) if count > 86_400 => {
            "Historical data request for greater than 86400 seconds rejected."
        }
        ('d', count) if count > 365 => {
            "Historical data requests for durations longer than 365 days must be made in years."
        }
        ('W', count) if count > 52 => {
            "Historical data request for durations longer than 52 weeks must be made in years."
        }
        ('m', count) if count > 12 => {
            "Historical data request for durations longer than 12 months must be made in years."
        }
        _ => return Ok(()),
    };
    Err(why.to_string())
}

/// Build the query the venue reads, as the XML it expects.
pub fn build_query_xml(req: &HistoricalRequest) -> String {
    build_stretch_xml(req, &Stretch::whole(req))
}

/// The name a graph is known by in a query's id and in the service's answers:
/// the ticker at its venue, and the series.
pub(crate) fn graph_name(req: &HistoricalRequest) -> String {
    let exchange = match req.exchange.as_str() {
        "SMART" => "BEST",
        e => e,
    };
    format!("{}@{} {}", req.symbol, exchange, req.data_type.as_str())
}

/// Build the query for one stretch of a request's contract's id history.
pub(crate) fn build_stretch_xml(req: &HistoricalRequest, s: &Stretch) -> String {
    let exchange = match req.exchange.as_str() {
        "SMART" => "BEST",
        e => e,
    };
    let rth = if req.use_rth { "true" } else { "false" };
    let expired = if s.expired { "yes" } else { "no" };
    let tag = |name: &str, value: &Option<String>| {
        value.as_ref().map_or(String::new(), |v| format!("<{name}>{v}</{name}>"))
    };

    // keepUpToDate uses structured ;;-delimited ID required by CCP gateway parser.
    // One-shot uses simple ID (HMDS accepts it fine).
    let query_id = if req.keep_up_to_date {
        format!("{};;{};;1;;true;;0;;I", req.query_id, graph_name(req))
    } else {
        req.query_id.clone()
    };

    let (end_time, refresh_tag) = if req.keep_up_to_date {
        (s.end_time.clone(), "<refresh>5 secs</refresh>")
    } else {
        (s.end_time.clone().or_else(|| Some(req.end_time.clone())), "")
    };

    let query = |data: &str, id: &str| format!(
        "<Query>\
         <id>{id}</id>\
         {approx_step}\
         <useRTH>{rth}</useRTH>\
         <contractID>{con_id}</contractID>\
         <exchange>{exchange}</exchange>\
         <secType>{sec_type}</secType>\
         <expired>{expired}</expired>\
         <type>BarData</type>\
         <data>{data}</data>\
         {start_time}\
         {end_time}\
         {underlying}\
         {listing}\
         {cutoff}\
         {refresh}\
         {dur}\
         <step>{step}</step>\
         <source>API</source>\
         {live}\
         <needTotalValue>false</needTotalValue>\
         <wholeDays>false</wholeDays>\
         <delay>auto</delay>\
         </Query>",
        approx_step = tag("approxStep", &s.approx_step),
        con_id = s.con_id,
        sec_type = req.sec_type,
        start_time = tag("startTime", &s.start_time),
        end_time = tag("endTime", &end_time),
        underlying = tag("histUnderlying", &s.underlying),
        listing = tag("histListExch", &s.listing),
        cutoff = tag("cutoffDate", &s.cutoff_date),
        dur = tag("timeLength", &s.time_length),
        step = req.bar_size.as_str(),
        refresh = refresh_tag,
        live = tag("liveContractID", &s.live_con_id.map(|id| id.to_string())),
    );

    // A name that is a pair goes out as a gateway sends it: one query per
    // side, each under an id of its own, so the two answers come back told
    // apart and are folded into the pair.
    let queries = match req.data_type.paired_series() {
        Some(second) => format!(
            "{}{}",
            query(req.data_type.as_str(), &query_id),
            query(second, &pair_id_under(req, second)),
        ),
        None => query(req.data_type.as_str(), &query_id),
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListOfQueries>{queries}</ListOfQueries>",
    )
}

/// The name the second query of a pair goes out under in its id: the query's
/// own id with the series it asks named beside it, as a gateway names the two
/// apart.
fn pair_id_under(req: &HistoricalRequest, second: &str) -> String {
    let exchange = match req.exchange.as_str() {
        "SMART" => "BEST",
        e => e,
    };
    format!("{};;{}@{} {}", req.query_id, req.symbol, exchange, second)
}

/// The id the second query of a pair goes out under, which its answer comes
/// back naming — `None` where the request is a single series. The engine
/// holds the number against both ids, so either answer finds its request.
pub(crate) fn pair_query_id(req: &HistoricalRequest) -> Option<String> {
    req.data_type.paired_series().map(|second| pair_id_under(req, second))
}

/// The two series of a paired request delivered as the answer states them:
/// one bar for every stamp both sides answered, opening at the bid side's own
/// average of the bar, at the highest the ask side reached, down to the
/// lowest the bid side reached, and closing at the ask side's own average.
/// A stamp only one side answered has no pair to state and is not delivered.
///
/// The pair itself trades nothing, so it states no volume, no average and no
/// count: each of those fields says none the way the venue's own answers say
/// none for a series that has them not.
///
/// Both sides are read in the order the venue stamped them, which the held
/// pages are put in before they reach here.
pub(crate) fn merge_pair(
    bid: &[HistoricalBar], ask: &[HistoricalBar],
) -> Vec<HistoricalBar> {
    let mut out = Vec::with_capacity(bid.len().min(ask.len()));
    let mut rest_bid = bid.iter();
    let mut rest_ask = ask.iter();
    let (mut next_bid, mut next_ask) = (rest_bid.next(), rest_ask.next());
    while let (Some(at_bid), Some(at_ask)) = (next_bid, next_ask) {
        match at_bid.time.cmp(&at_ask.time) {
            std::cmp::Ordering::Equal => {
                out.push(HistoricalBar {
                    end: if at_bid.end.is_empty() { at_ask.end.clone() } else { at_bid.end.clone() },
                    time: at_bid.time.clone(),
                    open: at_bid.wap,
                    high: at_ask.high,
                    low: at_bid.low,
                    close: at_ask.wap,
                    volume: -1,
                    wap: -1.0,
                    count: 0,
                });
                next_bid = rest_bid.next();
                next_ask = rest_ask.next();
            }
            std::cmp::Ordering::Less => next_bid = rest_bid.next(),
            std::cmp::Ordering::Greater => next_ask = rest_ask.next(),
        }
    }
    out
}

/// The query for one stretch of a contract's id history, where it differs from
/// the request it is asked for.
///
/// A gateway asks a request whose contract traded under more than one id, or
/// more than one ticker or listing, as one query per stretch: each under the id
/// it traded as, bounded to its days, the newest first.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Stretch {
    /// The id it is asked under.
    pub(crate) con_id: u32,
    /// The id the caller asked under, stated where it is asked under another.
    pub(crate) live_con_id: Option<u32>,
    /// Whether it is asked as a contract that has expired.
    pub(crate) expired: bool,
    /// Where it starts, where it is bounded at a start of its own.
    pub(crate) start_time: Option<String>,
    /// Where it ends, where that is not where the request ends.
    pub(crate) end_time: Option<String>,
    /// How far back it reaches, where it states no start.
    pub(crate) time_length: Option<String>,
    /// The first day of the id's that it is asked from.
    pub(crate) cutoff_date: Option<String>,
    /// The ticker it traded under, where the history names one.
    pub(crate) underlying: Option<String>,
    /// The listing it traded under, where the history names one and a
    /// gateway does not leave it out.
    pub(crate) listing: Option<String>,
    /// The step the first answer stated, on every stretch asked after it.
    pub(crate) approx_step: Option<String>,
}

impl Stretch {
    /// The request asked whole, under the id it names.
    pub(crate) fn whole(req: &HistoricalRequest) -> Self {
        Self {
            con_id: req.con_id,
            expired: req.include_expired,
            time_length: Some(req.duration.clone()),
            ..Default::default()
        }
    }
}

/// The listings a gateway leaves out of a stretch's query where the logon
/// names none of its own.
const NO_LISTING: [&str; 3] = ["BASKET", "VALUE", "CORPACT"];

/// The units a time length is stated in, in the order a gateway tries them on
/// the end of the length, and how long each is. The five a request may state:
/// the shape a gateway reads the duration against names no others.
const LENGTH_UNITS: [(&str, i64); 5] = [
    ("S", 1_000), ("d", 86_400_000), ("W", 604_800_000), ("m", 2_678_400_000), ("y", 31_536_000_000),
];

/// A time length read: how many, and of which unit.
fn read_length(length: &str) -> Option<(i64, (&'static str, i64))> {
    let unit = LENGTH_UNITS.iter().copied().find(|(suffix, _)| length.ends_with(suffix))?;
    let count = length.split_once(' ').map_or(length, |(count, _)| count).parse().ok()?;
    Some((count, unit))
}

/// The bars a series of a day or longer still wants once the newest stretch
/// is in, and what a later stretch is asked with because of them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BarsWanted {
    /// How many bars are still to come.
    pub(crate) left: i64,
    /// The day the series reaches back to, which the last stretch is cut at.
    pub(crate) from: String,
    /// The unit the request's time length is stated in, for the length a
    /// later stretch is asked with.
    unit: (&'static str, i64),
    /// How long one bar is counted as, in milliseconds.
    bar: i64,
}

impl BarsWanted {
    /// The length a stretch is asked with for the bars still wanted, stated in
    /// the request's own unit and rounded up: a day's bars counted in calendar
    /// days, seven for every five.
    pub(crate) fn length(&self) -> String {
        let (suffix, ms) = self.unit;
        let mut wanted = self.left * self.bar;
        if self.bar == 86_400_000 {
            wanted = wanted * 7 / 5;
        }
        format!("{} {suffix}", (wanted + ms - 1) / ms)
    }
}

/// The stretches a request is asked along, the bars it still wants once the
/// first is in, and the first days whose week, month, quarter or year is
/// joined.
pub(crate) type Plan = (Vec<Stretch>, Option<BarsWanted>, Vec<String>);

/// A request asked along its contract's id history: one stretch per id,
/// ticker or listing the history names within the request, newest first; for a
/// series of a day or longer asked along more than one, the bars it still
/// wants; and for a week or longer asked along more than one, the first days
/// of the stretches, whose weeks, months, quarters or years are one bar joined
/// from both sides.
///
/// Each is asked under the id it traded as, naming the one the caller asked
/// under where that is another. The newest is asked as the request is, from
/// the day it began. Each older one is bounded to its own days, from where the
/// request reaches back to or the day it began, whichever is later, to the day
/// after its last or the request's end, whichever is earlier, and is asked as
/// expired where its id is not the caller's. One whose days all fall outside
/// the request is not asked. A series asked along more than one stretch counts
/// its bars as they arrive: each later stretch is asked for the bars still
/// wanted, back from where it ends, and none once they are all in.
pub(crate) fn along(
    req: &HistoricalRequest, history: &[crate::control::adjustments::IdStretch],
) -> Result<Plan, String> {
    let unreadable = || format!(
        "a request ending {:?} and reaching back {:?} cannot be bounded along the \
         contract's id history",
        req.end_time, req.duration,
    );
    let ms_day = 86_400_000;
    let length = read_length(&req.duration);
    // Where the request ends, and how far back it reaches: a length in days
    // counted in calendar days, as a gateway counts it.
    let end = || -> Result<i64, String> {
        let end = crate::protocol::datetime::request_end(&req.end_time).ok_or_else(unreadable)?;
        Ok(end.timestamp().as_millisecond())
    };
    let back = || -> Result<i64, String> {
        let (count, (suffix, ms)) = length.ok_or_else(unreadable)?;
        let count = if suffix == "d" { (count as f64 * 1.4523809523809523) as i64 } else { count };
        Ok(count * ms)
    };
    let end_day = || end().map(|end| end.div_euclid(ms_day) * ms_day);
    // On GMT, as a gateway states both.
    let at = |ms: i64| {
        jiff::Timestamp::from_millisecond(ms).map_or_else(
            |_| String::new(),
            |at| at.to_zoned(jiff::tz::TimeZone::UTC).strftime("%Y%m%d-%H:%M:%S").to_string(),
        )
    };
    let day_of = |ms: i64| at(ms).chars().take(8).collect::<String>();
    let day_ms = |day: &str| crate::protocol::datetime::ib_datetime_to_unix_millis(&format!("{day}-00:00:00"));
    let dated = |day: &str| day.len() == 8 && day_ms(day).is_some();
    // How long one bar is counted as, for a bar of a day or more: a month is
    // counted as thirty-one days, as a gateway counts it.
    let bar = match req.bar_size {
        BarSize::Day1 => Some(ms_day),
        BarSize::Week1 => Some(7 * ms_day),
        BarSize::Month1 => Some(31 * ms_day),
        BarSize::Month3 => Some(93 * ms_day),
        BarSize::Year1 => Some(365 * ms_day),
        _ => None,
    };
    let joining = bar.is_some_and(|bar| bar > ms_day) && history.len() > 1;

    let mut out = Vec::new();
    let mut joins = Vec::new();
    for (n, stretch) in history.iter().enumerate() {
        if n >= 1 && (stretch.end == "-1" || day_of(end_day()? - back()?) > stretch.end) {
            continue;
        }
        if dated(&stretch.start) && stretch.start > day_of(end_day()?) {
            continue;
        }
        if joining && dated(&stretch.start) {
            joins.push(stretch.start.clone());
        }
        let moved = stretch.con_id != req.con_id;
        let mut s = Stretch {
            con_id: stretch.con_id,
            live_con_id: moved.then_some(req.con_id),
            expired: req.include_expired || (n >= 1 && moved),
            time_length: Some(req.duration.clone()),
            cutoff_date: dated(&stretch.start).then(|| stretch.start.clone()),
            underlying: (!stretch.symbol.is_empty()).then(|| stretch.symbol.clone()),
            listing: (!stretch.exchange.is_empty() && !NO_LISTING.contains(&stretch.exchange.as_str()))
                .then(|| stretch.exchange.clone()),
            ..Default::default()
        };
        if n >= 1 && day_of(end_day()?) > stretch.end {
            let reach = end()? - back()?;
            let from = day_ms(&stretch.start).map_or(reach, |start| start.max(reach));
            let to = (day_ms(&stretch.end).ok_or_else(unreadable)? + ms_day).min(end()?);
            if from >= to {
                continue;
            }
            s.start_time = Some(at(from));
            s.end_time = Some(at(to));
            s.time_length = None;
        }
        out.push(s);
    }
    let wanted = match (bar, length) {
        (Some(bar), Some((count, unit))) if history.len() > 1 && count * unit.1 / bar > 1 => {
            Some(BarsWanted { left: count * unit.1 / bar, from: day_of(end()? - back()?), unit, bar })
        }
        _ => None,
    };
    Ok((out, wanted, joins))
}

/// The week, the month, the quarter or the year a day falls in, for bars of
/// that size, as a gateway groups them: a week is named by its Monday, a
/// Sunday by the Monday before it, a month by its first day, a quarter by the
/// first day of its first month and a year by the first of January.
pub(crate) fn period_of(bar_size: BarSize, day: &str) -> Option<String> {
    let date = jiff::civil::Date::strptime("%Y%m%d", day.get(..8)?).ok()?;
    let first = match bar_size {
        BarSize::Week1 => {
            date.checked_sub(jiff::Span::new().days(i64::from(date.weekday().to_monday_zero_offset()))).ok()?
        }
        BarSize::Month1 => date.first_of_month(),
        BarSize::Month3 => {
            jiff::civil::Date::new(date.year(), (date.month() - 1) / 3 * 3 + 1, 1).ok()?
        }
        BarSize::Year1 => jiff::civil::Date::new(date.year(), 1, 1).ok()?,
        _ => return None,
    };
    Some(first.strftime("%Y%m%d").to_string())
}

/// Join the bars of a week, a month, a quarter or a year that more than one
/// stretch answered for into one, as a gateway joins them once the series is
/// whole: every run of bars falling in the period one of `starts` falls in.
/// The joined bar opens where the first opened and closes where the last
/// closed, spans both, takes the highest high and the lowest low, and sums
/// the volume and the count; its average is weighted by volume.
pub(crate) fn join_periods(
    bars: Vec<HistoricalBar>, starts: &[String], bar_size: BarSize,
) -> Vec<HistoricalBar> {
    let joined: Vec<String> = starts.iter().filter_map(|day| period_of(bar_size, day)).collect();
    let mut out: Vec<HistoricalBar> = Vec::with_capacity(bars.len());
    let mut last_period = None;
    for bar in bars {
        let period = period_of(bar_size, &bar.time).filter(|p| joined.contains(p));
        match (out.last_mut(), &period) {
            (Some(held), Some(period)) if last_period.as_ref() == Some(period) => {
                let volume = held.volume + bar.volume;
                // A week or a month with no volume keeps the first part's
                // average.
                if volume != 0 {
                    held.wap = (held.wap * held.volume as f64 + bar.wap * bar.volume as f64) / volume as f64;
                }
                held.volume = volume;
                held.count = held.count.saturating_add(bar.count);
                held.high = held.high.max(bar.high);
                held.low = held.low.min(bar.low);
                held.close = bar.close;
                held.end = bar.end;
            }
            _ => out.push(bar),
        }
        last_period = period;
    }
    out
}

/// Build a historical data query message.
pub fn build_historical_request(req: &HistoricalRequest, seq: u32) -> Vec<u8> {
    let xml = build_query_xml(req);
    fix::fix_build(
        &[
            (fix::TAG_MSG_TYPE, "W"),
            (TAG_HISTORICAL_XML, &xml),
        ],
        seq,
    )
}

/// Build a cancellation message for a real-time bar subscription.
pub fn build_cancel_request(ticker_id: &str, seq: u32) -> Vec<u8> {
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <ListOfCancelQueries>\
         <CancelQuery>\
         <id>ticker:{ticker_id}</id>\
         </CancelQuery>\
         </ListOfCancelQueries>",
    );
    fix::fix_build(
        &[
            (fix::TAG_MSG_TYPE, "Z"),
            (TAG_HISTORICAL_XML, &xml),
        ],
        seq,
    )
}

/// The smallest price move the venue states for a contract, where it states one.
///
/// Every price on the contract is decoded against this, so the wrong one does
/// not fail — it scales every price by a constant and reports the result as
/// the venue's. A penny is the right answer for a US share and wrong for a
/// currency pair, a future, and anything quoted in yen — so where the venue
/// states none, this states none, and the bars are left unread.
///
/// The venue states it on the definition, so a definition that does not is the
/// interesting case and says so rather than passing quietly as a share.
pub fn min_tick_of(xml_tag: &str, ticker_id: &str) -> Option<f64> {
    match tag(xml_tag, "minTick").and_then(|s| s.parse::<f64>().ok()) {
        Some(tick) => Some(tick),
        None => {
            // Nothing is decoded without it. Prices in a bar are counted in
            // this unit, so choosing one decides every price in the answer:
            // a penny is right for a US share and wrong for everything that
            // moves in anything else, and the caller cannot tell which they
            // were handed. A bar that cannot be read is told; a bar read at a
            // unit nobody stated is a wrong price presented as a right one.
            log::warn!("no minTick stated for ticker {ticker_id}; its bars cannot be read");
            None
        }
    }
}

/// Parse a ResultSetBar XML response into bars.
pub fn parse_bar_response(xml: &str) -> Option<HistoricalResponse> {
    // Check for ResultSetBar
    if !xml.contains("<ResultSetBar>") {
        return None;
    }

    let query_id = tag(xml, "id").unwrap_or("").to_string();
    let timezone = tag(xml, "tz").unwrap_or("").to_string();
    let is_complete = tag(xml, "eoq").unwrap_or("false") == "true";

    let mut bars = Vec::new();
    let mut search_start = 0;

    while let Some(bar_start) = xml[search_start..].find("<Bar>") {
        let abs_start = search_start + bar_start;
        let bar_end = {
            // A row never closed is an answer cut short: the whole of it is
            // refused rather than what is in hand delivered as though
            // complete, which is how the histogram rows beside it are read.
            // Broken out of, a reply cut mid-row lost that row and everything
            // after it, and a short series arrived under the completeness the
            // reply's own end-of-query flag stated.
            xml[abs_start..].find("</Bar>")? + abs_start + 6
        };
        let bar_xml = &xml[abs_start..bar_end];

        // A price the bar does not state is not nought. Read that way a bar
        // whose close went missing is a crash to zero on a caller's chart, and
        // nothing in it says the number was never sent — which is the same
        // reason a bar whose unit nobody stated is not read either.
        let priced = |name: &str| -> Option<f64> {
            tag(bar_xml, name).and_then(|s| s.parse().ok())
        };
        let (Some(open), Some(high), Some(low), Some(close)) =
            (priced("open"), priced("high"), priced("low"), priced("close"))
        else {
            log::warn!("a bar states no open, high, low or close, so the series is not read");
            return None;
        };
        let bar = HistoricalBar {
            // A bar the venue aggregated is dated rather than timed: a week
            // and a month arrive as `date` and `endDate` where everything
            // shorter arrives as `time` and `endTime`. Read for `time` alone
            // they reached the caller with no date at all.
            time: tag(bar_xml, "time")
                .or_else(|| tag(bar_xml, "date"))
                .unwrap_or("")
                .to_string(),
            // And where it ends, under the same two spellings. Read under
            // neither, the bounds the venue states for a week or a month were
            // thrown away unread.
            end: tag(bar_xml, "endTime")
                .or_else(|| tag(bar_xml, "endDate"))
                .unwrap_or("")
                .to_string(),
            open,
            high,
            low,
            close,
            volume: tag(bar_xml, "volume")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            wap: tag(bar_xml, "weightedAvg")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            count: tag(bar_xml, "count")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        };
        bars.push(bar);
        search_start = bar_end;
    }

    Some(HistoricalResponse {
        query_id,
        timezone,
        bars,
        is_complete,
    })
}

/// The day each session open in a bar reply states it opens, in the order
/// stated: an answer below a day marks every session it covers with an open,
/// its reference day beside it.
pub(crate) fn session_opens(xml: &str) -> Vec<String> {
    xml.split("<Open>")
        .skip(1)
        .filter_map(|open| tag(open.split("</Open>").next()?, "refDate").map(str::to_string))
        .collect()
}

/// Extract the ticker ID from a ResultSetTickerId response (for real-time bar
/// subscriptions).
pub fn parse_ticker_id(xml: &str) -> Option<String> {
    if !xml.contains("<ResultSetTickerId>") {
        return None;
    }
    // The assignment comes back under either name: a bar subscription is
    // answered with `tickerId` and a tick-by-tick one with `rtTickerId`. The
    // second name is a fallback only — a tick-by-tick acknowledgement is taken
    // and answered before this is reached — but it costs nothing and the two
    // shapes are not otherwise told apart here.
    tag(xml, "tickerId")
        .or_else(|| tag(xml, "rtTickerId"))
        .map(|s| s.to_string())
}

/// What the venue calls a series asked for the earliest moment it holds.
///
/// A head timestamp is a query type of its own, and the venue's vocabulary for
/// it is not the bar one. Asked here for the option-exercise rate it answers
/// that it holds no data for the contract, naming the series back — the same
/// answer the tick query gives — where a bar query for the same name is
/// refused as a query type that does not take it. Read through the bar table
/// alone, the one name the two vocabularies differ on was refused here without
/// being asked.
pub fn head_timestamp_data_type(what_to_show: &str) -> Result<&'static str, String> {
    if what_to_show.eq_ignore_ascii_case("OPTION_EXERCISE_INTEREST_RATE") {
        return Ok("OptExInterestRate");
    }
    // The adjusted series begins where the raw trades it is folded from do,
    // and a gateway asks for the earliest trade to answer it.
    if what_to_show_is_adjusted(what_to_show) {
        return Ok(BarDataType::Trades.as_str());
    }
    BarDataType::from_api_str(what_to_show).map(|series| series.as_str())
}

/// Parameters for a head timestamp request.
#[derive(Debug, Clone)]
pub struct HeadTimestampRequest {
    /// This query's own name, which leads the id it goes out under.
    ///
    /// Two callers asking about one contract on one venue for one series
    /// described the same request, so the id built from that description was
    /// the same for both — and the answers were matched to whichever of them
    /// sat first in the list. One got the other's answer, a duplicate answered
    /// once left the second waiting for ever, and one caller's cancel stopped
    /// the other's query at the venue. The histogram beside this one is led by
    /// its own name for exactly this reason.
    pub query_id: String,
    /// The venue's id for the contract.
    pub con_id: u32,
    /// What kind of contract it is, as the venue names it.
    pub sec_type: String,
    /// Which venue to answer for.
    pub exchange: String,
    /// Which series is wanted, under the name the venue knows it by.
    pub data_type: &'static str,
    /// Whether to count only regular trading hours.
    pub use_rth: bool,
    /// Whether an expired contract is meant, stated as the bar query states it.
    pub include_expired: bool,
}

/// Parsed head timestamp response.
#[derive(Debug, Clone)]
pub struct HeadTimestampResponse {
    /// The earliest moment the venue holds data for.
    pub head_timestamp: String,
    /// The zone the times are stated in.
    pub timezone: String,
}

/// The id a head-timestamp query goes out under, which is what its answer
/// comes back naming.
///
/// Built from the request itself, so the caller that sent it can be found from
/// the reply rather than from the order the replies happen to arrive in.
pub fn head_timestamp_query_id(req: &HeadTimestampRequest) -> String {
    let exchange = match req.exchange.as_str() {
        "SMART" => "BEST",
        e => e,
    };
    let rth = if req.use_rth { "true" } else { "false" };
    format!("{};;{}@{} {};;0;;{};;0;;U",
        req.query_id, req.con_id, exchange, req.data_type, rth)
}

/// Build the XML query for a head timestamp request.
pub fn build_head_timestamp_xml(req: &HeadTimestampRequest) -> String {
    let exchange = match req.exchange.as_str() {
        "SMART" => "BEST",
        e => e,
    };
    let rth = if req.use_rth { "true" } else { "false" };
    let id = head_timestamp_query_id(req);

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <ListOfQueries>\
         <Query>\
         <id>{id}</id>\
         <useRTH>{rth}</useRTH>\
         <expired>{expired}</expired>\
         <contractID>{con_id}</contractID>\
         <exchange>{exchange}</exchange>\
         <secType>{sec_type}</secType>\
         <type>TickHeadTimeStamp</type>\
         <data>{data}</data>\
         <step>-1</step>\
         <source>API</source>\
         <needTotalValue>false</needTotalValue>\
         <wholeDays>false</wholeDays>\
         <delay>auto</delay>\
         </Query>\
         </ListOfQueries>",
        expired = if req.include_expired { "yes" } else { "no" },
        con_id = req.con_id,
        sec_type = req.sec_type,
        data = req.data_type,
    )
}

/// Map whatToShow to data type.
/// What the venue calls a tick series, from what a caller calls it.
///
/// A name this does not know is refused rather than turned into trades.
/// Falling back to trades answers a misspelled `BID`, or the venue's
/// interest-rate series, with option prints and reports nothing.
pub fn tick_data_type(what_to_show: &str) -> Result<&'static str, String> {
    Ok(match what_to_show.to_uppercase().as_str() {
        "" | "TRADES" => "AllLast",
        "MIDPOINT" => "MidPoint",
        "BID_ASK" => "BidAsk",
        // Trades as the venue aggregates them, which it serves on the tick
        // query as well as the bar one.
        "AGGTRADES" => "AggLast",
        // The rate the venue prices options at. A tick type, not a bar one:
        // asked for as bars the venue answers that the query type is not
        // supported for it, and names the tick type back. Every window asked
        // for so far has come back empty, so what it takes and what it holds
        // are two different questions and only the first is answered here.
        "OPTION_EXERCISE_INTEREST_RATE" => "OptExInterestRate",
        other => {
            return Err(format!(
                "Unsupported what_to_show '{other}' for historical ticks: expected TRADES, \
                 MIDPOINT, BID_ASK, AGGTRADES or OPTION_EXERCISE_INTEREST_RATE",
            ));
        }
    })
}

/// What a historical-tick window has to state to be askable.
///
/// The query this client sends is bounded at its end and counts back from
/// there. Writing a start into that same field returns the ticks before the
/// moment rather than after it: the right number of records, off the wrong
/// side of the clock, with nothing in them to say so.
pub fn validate_tick_window(start_date_time: &str, end_date_time: &str) -> Result<(), String> {
    match (start_date_time.is_empty(), end_date_time.is_empty()) {
        // One end and a count is what the venue serves an API client, and it
        // is the venue that says so — asked with neither it answers "2 out of
        // startTime/endTime/timeLength parameters have to be specified", and
        // with both and no count "Times and Sales queries not length based not
        // allowed from API".
        (true, true) => Err(
            "historical ticks are asked for from one end and counted from there, and this \
             request names neither. Give start_date_time for the ticks after a moment, or \
             end_date_time for the ones before it."
                .to_string(),
        ),
        (false, false) => Err(format!(
            "historical ticks are asked for from one end and counted from there, and this \
             request names both ({start_date_time} and {end_date_time}). Give one of them \
             and the count says how far it reaches.",
        )),
        _ => Ok(()),
    }
}

/// Build the XML query for a historical ticks request.
///
/// Uses `<type>TickData</type>`, `<step>ticks</step>`, `<timeLength>{N}
/// t</timeLength>`.
#[allow(clippy::too_many_arguments)]
pub fn build_tick_query_xml(
    query_id: &str, con_id: i64, start_date_time: &str, end_date_time: &str,
    number_of_ticks: u32, what_to_show: &str, use_rth: bool,
    sec_type: &str, exchange: &str, include_expired: bool, ignore_size: bool,
) -> String {
    let expired = if include_expired { "yes" } else { "no" };
    // Stated from the contract, and left unstated where the contract does
    // not state it. Assuming a BEST-routed US stock describes every other
    // kind of contract wrongly, and a description stated here is one the
    // venue reads. The contract id identifies the contract exactly, so an
    // empty field asks about it rather than about something else.
    let rth = if use_rth { "true" } else { "false" };
    let data = tick_data_type(what_to_show).unwrap_or("AllLast");

    // Both bounds, each in the field that carries it. The venue holds a start
    // and an end apart and the reference client passes both, so a request
    // naming only a start is one it serves; this used to refuse that, having
    // once written the start into the end's field and asked for the ticks
    // before the moment the caller wanted the ticks after — the answer looked
    // right and covered the wrong side of the clock. The field, not the
    // request, was the trouble.
    //
    // One end and a count, which is what the venue serves an API client. It
    // says so itself when asked otherwise: naming neither end is answered
    // "2 out of startTime/endTime/timeLength parameters have to be specified",
    // and naming both without a count "Times and Sales queries not length
    // based not allowed from API". Either end will do — a start was refused
    // here before it was ever sent, and the venue answers one with the ticks
    // after it.
    let time_tag = if start_date_time.is_empty() {
        format!("<endTime>{end_date_time}</endTime>")
    } else {
        format!("<startTime>{start_date_time}</startTime>")
    };
    let length_tag = format!("<timeLength>{number_of_ticks} t</timeLength>");
    // The filter that leaves out a change moving only the size, where a
    // gateway writes one: on bid/ask when the caller asks for it, and on
    // every midpoint query whatever the caller asked. Trades carry none.
    let filter = if data == "MidPoint" || (data == "BidAsk" && ignore_size) {
        "<filter><ignoreSize>true</ignoreSize></filter>"
    } else {
        ""
    };

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <ListOfQueries>\
         <Query>\
         <id>{query_id}</id>\
         <useRTH>{rth}</useRTH>\
         <contractID>{con_id}</contractID>\
         <exchange>{exchange}</exchange>\
         <secType>{sec_type}</secType>\
         <expired>{expired}</expired>\
         <type>TickData</type>\
         <data>{data}</data>\
         {time_tag}\
         {length_tag}\
         <step>ticks</step>\
         <source>API</source>\
         <wholeDays>true</wholeDays>\
         <delay>auto</delay>\
         {filter}\
         </Query>\
         </ListOfQueries>",
    )
}

/// The shape a reply of this kind carries, with nothing in it.
///
/// Stated beside the parse that fills it, because the two have to agree: a
/// series that cannot be read is still ended, and it is ended under the kind
/// the caller asked for. Ended under another, the answer goes to a callback
/// nobody is waiting on and the caller waits out its whole deadline for a
/// completion it was already sent.
pub fn no_ticks_of_the_kind(what_to_show: &str) -> crate::types::HistoricalTickData {
    match what_to_show.to_uppercase().as_str() {
        "BID_ASK" => crate::types::HistoricalTickData::BidAsk(Vec::new()),
        "MIDPOINT" | "OPTION_EXERCISE_INTEREST_RATE" => {
            crate::types::HistoricalTickData::Midpoint(Vec::new())
        }
        _ => crate::types::HistoricalTickData::Last(Vec::new()),
    }
}

/// Whether a row's marks carry one of the venue's codes.
///
/// The venue states them as a set of letters in one element, and a caller is
/// owed what they say: read nowhere, a print the venue marked unreported, or a
/// quote it marked past the limit, reached the caller marked as neither — which
/// is a statement about the print, not the absence of one.
fn flagged(tick_xml: &str, code: &str) -> bool {
    tag(tick_xml, "flags")
        .is_some_and(|flags| flags.split(|c: char| !c.is_ascii_alphabetic()).any(|f| f == code))
}

/// Parse a ResultSetTick XML response into historical tick data.
pub fn parse_tick_response(xml: &str, what_to_show: &str) -> Option<(String, crate::types::HistoricalTickData, bool)> {
    if !xml.contains("<ResultSetTick>") {
        return None;
    }

    let query_id = tag(xml, "id").unwrap_or("").to_string();
    let is_complete = tag(xml, "eoq").unwrap_or("false") == "true";

    let upper = what_to_show.to_uppercase();
    let mut search_start = 0;

    match upper.as_str() {
        "BID_ASK" => {
            let mut ticks = Vec::new();
            while let Some(tick_pos) = xml[search_start..].find("<Tick>") {
                let abs = search_start + tick_pos;
                let end = {
                    // A row never closed is an answer cut short: the whole of
                    // it is refused rather than what is in hand delivered as
                    // though complete, which is how the histogram rows beside
                    // it are read. Broken out of, a reply cut mid-row lost
                    // that row and everything after it, and a short series
                    // arrived under the completeness the reply's own
                    // end-of-query flag stated.
                    xml[abs..].find("</Tick>")? + abs + 7
                };
                let t = &xml[abs..end];
                // Named the way the answer names them. The caller-facing
                // message for one of these rows spells its fields the other
                // way round, and read for those spellings every row of every
                // series arrived with a bid, an ask and two sizes of nought —
                // a full run of quotes at zero, timed correctly and marked
                // complete, that the venue never stated.
                //
                // And a row that states none of them is not a quote at zero
                // either: the series is refused, the way a bar with no open,
                // high, low or close is.
                let stated = |name: &str| -> Option<f64> {
                    tag(t, name).and_then(|s| s.parse().ok())
                };
                let (Some(bid_price), Some(ask_price), Some(bid_size), Some(ask_size)) = (
                    stated("bidPrice"), stated("askPrice"),
                    stated("bidSize"), stated("askSize"),
                ) else {
                    log::warn!(
                        "a quote states no bid, ask or size, so the series is not read",
                    );
                    return None;
                };
                ticks.push(crate::types::HistoricalTickBidAsk {
                    time: tag(t, "time").unwrap_or("").to_string(),
                    bid_price,
                    ask_price,
                    bid_size,
                    ask_size,
                    // The same marks, on the two sides of a quote.
                    bid_past_low: flagged(t, "BH"),
                    ask_past_high: flagged(t, "AH"),
                });
                search_start = end;
            }
            Some((query_id, crate::types::HistoricalTickData::BidAsk(ticks), is_complete))
        }
        // A rate is a value with a moment, and nothing else. Read through
        // the trade decoder it arrives as a print, with a size and a venue it
        // never had. No trade is involved in this series.
        "MIDPOINT" | "OPTION_EXERCISE_INTEREST_RATE" => {
            let mut ticks = Vec::new();
            while let Some(tick_pos) = xml[search_start..].find("<Tick>") {
                let abs = search_start + tick_pos;
                let end = {
                    // A row never closed is an answer cut short: the whole of
                    // it is refused rather than what is in hand delivered as
                    // though complete, which is how the histogram rows beside
                    // it are read. Broken out of, a reply cut mid-row lost
                    // that row and everything after it, and a short series
                    // arrived under the completeness the reply's own
                    // end-of-query flag stated.
                    xml[abs..].find("</Tick>")? + abs + 7
                };
                let t = &xml[abs..end];
                ticks.push(crate::types::HistoricalTickMidpoint {
                    time: tag(t, "time").unwrap_or("").to_string(),
                    price: tag(t, "price").and_then(|s| s.parse().ok()).unwrap_or(0.0),
                });
                search_start = end;
            }
            Some((query_id, crate::types::HistoricalTickData::Midpoint(ticks), is_complete))
        }
        _ => {
            // TRADES / AllLast
            let mut ticks = Vec::new();
            while let Some(tick_pos) = xml[search_start..].find("<Tick>") {
                let abs = search_start + tick_pos;
                let end = {
                    // A row never closed is an answer cut short: the whole of
                    // it is refused rather than what is in hand delivered as
                    // though complete, which is how the histogram rows beside
                    // it are read. Broken out of, a reply cut mid-row lost
                    // that row and everything after it, and a short series
                    // arrived under the completeness the reply's own
                    // end-of-query flag stated.
                    xml[abs..].find("</Tick>")? + abs + 7
                };
                let t = &xml[abs..end];
                // Named the way the answer names them, which is shorter than
                // the field each one fills. Read for the field names, the venue
                // that printed a trade and what it noted about it came back
                // empty on every row of every series — and empty is what a
                // print the venue left unattributed looks like, so nothing said
                // the fields had never been read.
                let stated = |name: &str| -> Option<f64> {
                    tag(t, name).and_then(|s| s.parse().ok())
                };
                let (Some(price), Some(size)) = (stated("price"), stated("size")) else {
                    log::warn!("a print states no price or size, so the series is not read");
                    return None;
                };
                ticks.push(crate::types::HistoricalTickLast {
                    time: tag(t, "time").unwrap_or("").to_string(),
                    price,
                    size,
                    exchange: tag(t, "exch").unwrap_or("").to_string(),
                    special_conditions: tag(t, "cond").unwrap_or("").to_string(),
                    // What the venue marked the print with. Published as all
                    // false where it was never read, which is a statement about
                    // the print rather than the absence of one.
                    past_limit: flagged(t, "H"),
                    unreported: flagged(t, "U"),
                });
                search_start = end;
            }
            Some((query_id, crate::types::HistoricalTickData::Last(ticks), is_complete))
        }
    }
}

/// Build the XML subscription for real-time 5-second bars.
pub fn build_realtime_bar_xml(
    query_id: &str, con_id: i64, what_to_show: &str, use_rth: bool,
    sec_type: &str, exchange: &str,
) -> String {
    // Stated from the contract rather than assumed. A stock routed BEST was
    // the only shape this ever described, so a request for anything else — an
    // FX pair on IDEALPRO, a future on its own venue — went out saying it was
    // a US stock and came back untyped.

    let rth = if use_rth { "true" } else { "false" };
    // Through the one place that knows these names. Spelled out again here,
    // the midpoint was "Midpoint" in two of the three and "MidPoint" in the
    // third — and the venue, which only takes the third, answered the other
    // two with "no historical market data", which reads as a series that does
    // not exist rather than a name that is misspelled.
    // Refused at the request, so nothing reaches here that this does not know.
    // Falling back to trades sent a different series than the one asked for,
    // and the bars that came back read as the ones the caller wanted.
    let data = BarDataType::from_api_str(what_to_show)
        .map(|kind| kind.as_str())
        .unwrap_or("Last");

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <ListOfQueries>\
         <Query>\
         <id>{query_id}</id>\
         <useRTH>{rth}</useRTH>\
         <contractID>{con_id}</contractID>\
         <exchange>{exchange}</exchange>\
         <secType>{sec_type}</secType>\
         <type>BarData</type>\
         <data>{data}</data>\
         <refresh>5 secs</refresh>\
         <step>5 secs</step>\
         <source>API</source>\
         <needTotalValue>false</needTotalValue>\
         <wholeDays>false</wholeDays>\
         </Query>\
         </ListOfQueries>",
    )
}

/// Decode a real-time bar binary payload.
///
/// Uses LSB-first bit reader with 4-byte group reversal.
/// Returns (low, open, high, close, volume, wap, count) or None.
pub fn decode_bar_payload(
    payload: &[u8],
    min_tick: f64,
    size_tick: f64,
) -> Option<crate::types::RealTimeBar> {
    if payload.is_empty() {
        return None;
    }

    // Reverse byte order within 4-byte groups
    let mut reordered = Vec::with_capacity(payload.len());
    for chunk in payload.chunks(4) {
        for &b in chunk.iter().rev() {
            reordered.push(b);
        }
    }

    let data = &reordered;
    let mut pos: usize = 0; // bit position

    // A read past the end of the payload takes zeroes, and so does every
    // field after it. Unrecorded, a payload cut anywhere decodes into a bar
    // of plausible zeroes indistinguishable from one the venue sent.
    let overran = std::cell::Cell::new(false);
    let read_bits = |pos: &mut usize, n: usize| -> u32 {
        let mut val: u32 = 0;
        for i in 0..n {
            let byte_idx = *pos / 8;
            let bit_idx = *pos % 8;
            if byte_idx < data.len() {
                val |= (((data[byte_idx] >> bit_idx) & 1) as u32) << i;
            } else {
                overran.set(true);
            }
            *pos += 1;
        }
        val
    };

    // 4 bits padding
    read_bits(&mut pos, 4);

    // Count: 1-bit flag selects width
    let count = if read_bits(&mut pos, 1) == 1 {
        read_bits(&mut pos, 8) as i32
    } else {
        // A stated count past what the width every surface reports it in
        // carries is not one this client can hold, and the payload is refused.
        //
        // Cast straight through, it reached the caller as a bar made by minus
        // two billion trades. Read as a bar stating no count — which is what
        // the other decoder of this field does with an unreadable one — it is
        // worse: the count decides whether the fields after it are here at
        // all, so calling it nought skips bits the sender wrote, and every
        // field behind it is read from the wrong offset. The cursor ends short
        // rather than past the end, so the overrun guard below never fires and
        // the bar arrives complete, plausible, and made up.
        //
        // That decoder can hold its field alone because its fields do not
        // position each other. This one cannot, so it refuses the payload the
        // way it refuses one whose prices will not read.
        match i32::try_from(read_bits(&mut pos, 32)) {
            Ok(stated) => stated,
            Err(_) => return None,
        }
    };

    // Low price in ticks (31-bit signed)
    let low_ticks = read_bits(&mut pos, 31);
    // Sign-extended in a width that holds the intermediate. A 31-bit value with
    // its sign bit set is the raw value less 2^31, and that subtraction does
    // not fit an i32 — a bar with a low below zero, which a spread has, took
    // the process down.
    let low_ticks_signed = if low_ticks & (1 << 30) != 0 {
        (low_ticks as i64 - (1i64 << 31)) as i32
    } else {
        low_ticks as i32
    };
    let low = low_ticks_signed as f64 * min_tick;

    let (open, high, close, wap_sum);
    if count > 1 {
        // Delta width: 1-bit flag
        let width = if read_bits(&mut pos, 1) == 1 { 5 } else { 32 };
        let d_open = read_bits(&mut pos, width);
        let d_high = read_bits(&mut pos, width);
        let d_close = read_bits(&mut pos, width);

        open = low + d_open as f64 * min_tick;
        high = low + d_high as f64 * min_tick;
        close = low + d_close as f64 * min_tick;

        // WAP sum: 1-bit flag selects width
        wap_sum = if read_bits(&mut pos, 1) == 1 {
            read_bits(&mut pos, 18) as f64
        } else {
            read_bits(&mut pos, 32) as f64
        };
    } else {
        open = low;
        high = low;
        close = low;
        wap_sum = 0.0;
    }

    // Volume: 1-bit flag selects width. A count of the increment the venue
    // said this contract's sizes move in, the same as a size on the quote and
    // tick-by-tick streams — where it was left as a whole number, one
    // instrument reported two different volumes.
    let counted = if read_bits(&mut pos, 1) == 1 {
        read_bits(&mut pos, 16) as f64
    } else {
        read_bits(&mut pos, 32) as f64
    };
    // An increment of nothing would zero every volume on the contract, which
    // reads as a bar nobody traded rather than as the missing increment it is.
    // Both writers guarantee a positive one today; this is what happens if a
    // third does not.
    let volume = counted * if size_tick > 0.0 { size_tick } else { 1.0 };

    // Divided by the count, not by the volume above it. The weighted sum is a
    // raw wire figure weighted by those same counts, so the two cancel; put
    // the scaled volume underneath it instead and the offset from the low
    // scales by the reciprocal of the increment — which on a contract counted
    // in hundred-millionths reads a sixty-thousand-dollar bar at fifty
    // million.
    let wap = if count > 1 && counted > 0.0 {
        low + wap_sum * min_tick / counted
    } else {
        low
    };

    if overran.get() {
        return None;
    }

    Some(crate::types::RealTimeBar {
        timestamp: 0, // filled by caller from message header
        open, high, low, close, volume, wap, count,
    })
}

/// Build the XML query for a historical schedule request.
///
/// Schedule requests use `<data>Schedule</data>` and
/// `<scheduleOnly>true</scheduleOnly>`
/// with `<type>BarData</type>`. Response is `<ResultSetSchedule>`.
pub fn build_schedule_xml(
    query_id: &str, con_id: i64, end_time: &str, duration: &str, use_rth: bool,
    sec_type: &str, exchange: &str,
) -> String {
    // The last of the query builders that described a US stock routed BEST
    // whatever contract it was asked about.

    let rth = if use_rth { "true" } else { "false" };

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <ListOfQueries>\
         <Query>\
         <id>{query_id}</id>\
         <useRTH>{rth}</useRTH>\
         <contractID>{con_id}</contractID>\
         <exchange>{exchange}</exchange>\
         <secType>{sec_type}</secType>\
         <type>BarData</type>\
         <data>Schedule</data>\
         <endTime>{end_time}</endTime>\
         <timeLength>{duration}</timeLength>\
         <step>1 day</step>\
         <scheduleOnly>true</scheduleOnly>\
         </Query>\
         </ListOfQueries>",
    )
}

/// Parse a ResultSetSchedule XML response into sessions.
pub fn parse_schedule_response(xml: &str) -> Option<crate::types::HistoricalScheduleResponse> {
    if !xml.contains("<ResultSetSchedule>") {
        return None;
    }

    let query_id = tag(xml, "id").unwrap_or("").to_string();
    let timezone = tag(xml, "tz").unwrap_or("").to_string();
    let start_date_time = tag(xml, "derivedStart").unwrap_or("").to_string();

    let mut sessions = Vec::new();
    let mut search_start = 0;

    // Parse Open/Close pairs into sessions
    while let Some(open_pos) = xml[search_start..].find("<Open>") {
        let abs_open = search_start + open_pos;
        let open_end = {
            // A row never closed is an answer cut short: the whole of it is
            // refused rather than what is in hand delivered as though
            // complete, which is how the histogram rows beside it are read.
            // Broken out of, a reply cut mid-row lost that row and everything
            // after it, and a short series arrived under the completeness the
            // reply's own end-of-query flag stated.
            xml[abs_open..].find("</Open>")? + abs_open + 7
        };
        let open_xml = &xml[abs_open..open_end];

        let open_time = tag(open_xml, "time").unwrap_or("").to_string();
        let ref_date = tag(open_xml, "refDate").unwrap_or("").to_string();

        // Find the matching Close
        let close_time = if let Some(close_pos) = xml[open_end..].find("<Close>") {
            let abs_close = open_end + close_pos;
            let close_end = {
                // A row never closed is an answer cut short: the whole of it
                // is refused rather than what is in hand delivered as though
                // complete, which is how the histogram rows beside it are
                // read. Broken out of, a reply cut mid-row lost that row and
                // everything after it, and a short series arrived under the
                // completeness the reply's own end-of-query flag stated.
                xml[abs_close..].find("</Close>")? + abs_close + 8
            };
            let close_xml = &xml[abs_close..close_end];
            search_start = close_end;
            tag(close_xml, "time").unwrap_or("").to_string()
        } else {
            search_start = open_end;
            String::new()
        };

        sessions.push(crate::types::ScheduleSession {
            ref_date,
            open_time,
            close_time,
        });
    }

    Some(crate::types::HistoricalScheduleResponse {
        query_id,
        timezone,
        start_date_time,
        // The venue derives both ends of what it covered and states both.
        // Only the start was read, and the end a caller got back was the one
        // it had asked for — so a request reaching past what the venue holds
        // was answered with its own timestamp as the coverage.
        end_date_time: tag(xml, "derivedEnd").unwrap_or("").to_string(),
        sessions,
    })
}

/// Parse a ResultSetHeadTimeStamp XML response.
pub fn parse_head_timestamp_response(xml: &str) -> Option<HeadTimestampResponse> {
    if !xml.contains("<ResultSetHeadTimeStamp>") {
        return None;
    }
    let head_timestamp = tag(xml, "headTS")?.to_string();
    let timezone = tag(xml, "tz").unwrap_or("").to_string();
    Some(HeadTimestampResponse { head_timestamp, timezone })
}

#[cfg(test)]
pub(crate) mod tests;
