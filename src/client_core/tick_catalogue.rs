//! The catalogue of series the venue knows, and the types each is legal for
//! in a quote request.
//!
//! A quote request states the series it wants beside the quote as a list of
//! numbers. A gateway reads the list whole: an entry that is not a number the
//! venue knows a series by, one it knows but the request's type may not ask
//! for, or a word that is no number at all refuses the whole request under
//! 321, in a standing text that names the legal series for the type. This is
//! that catalogue, whole: a partial table would refuse lists a gateway takes,
//! so the series no quote request may ask for are kept here, under their
//! refusal, beside the ones every type may ask for.
//!
//! A number the venue also answers an older number by resolves to the series
//! it names, and the older numbers are kept beside it. One entry — the credit
//! mark — is itself answered by an older number of the pl-price series: the
//! number 221 resolves to 232, and the entry registered under 221 is what the
//! catalogue states to a caller who asks what 221 is.

/// Which security types a series is legal for in a quote request.
#[derive(Clone, Copy)]
enum Legal {
    /// No quote request may ask for it, of any type. The venue knows the
    /// series; its own features ask for it where it is asked at all.
    Never,
    /// Every type may ask for it.
    Always,
    /// Legal where the session may read news.
    News,
    /// Legal for the listed types only.
    Types(&'static [&'static str]),
}

use Legal::{Always, Never, News, Types};

/// The types the dividends series is legal for.
const DIVIDENDS: &[&str] = &["STK", "FUT", "OPT", "IND", "FOP", "CFD", "SLB"];

/// The types the short-term volume series is legal for.
const SHORT_TERM_VOLUME: &[&str] =
    &["STK", "CFD", "OPT", "FOP", "WAR", "IOPT", "FUT", "FWD", "BOND", "BILL", "SLB", "CRYPTO"];

/// The one type the index premium series is legal for.
const INDEX: &[&str] = &["IND"];

/// The catalogue, ascending by the venue's number, as the legal list a
/// refusal states is rendered ascending: each series under its number, the
/// older number it also answers by where it has one (0 for none), the name
/// the legal list states it under, and the types it is legal for.
const CATALOGUE: &[(i32, i32, &str, Legal)] = &[
    (100, 0, "Option Volume", Always),
    (101, 0, "Option Open Interest", Always),
    (105, 0, "Average Opt Volume", Always),
    (106, 0, "impvolat", Always),
    (107, 0, "climpvlt", Never),
    (125, 0, "Bond analytic data", Never),
    (162, 0, "Index Future Premium", Types(INDEX)),
    (165, 0, "Misc. Stats", Always),
    (221, 220, "Creditman Mark Price", Always),
    (225, 0, "Auction", Always),
    (230, 0, "GreekPortfolioVWAP", Never),
    (232, 221, "Pl Price", Always),
    (233, 0, "RTVolume", Always),
    (236, 0, "inventory", Always),
    (247, 0, "Top_News", Never),
    (258, 47, "Fundamentals", Always),
    (266, 0, "Index Value by Futures", Never),
    (291, 0, "ivclose", Never),
    (292, 0, "Wide_news", News),
    (293, 0, "TradeCount", Always),
    (294, 0, "TradeRate", Always),
    (295, 0, "VolumeRate", Always),
    (317, 0, "UnderlyingFrozenPlPrice(fztuplprc)", Never),
    (318, 0, "LastRTHTrade", Always),
    (320, 0, "UnderlyingFrozenBidAsk(fzubidask)", Never),
    (375, 0, "RTTrdVolume", Always),
    (376, 0, "GenericRfq", Never),
    (386, 0, "CompanyEvents", Never),
    (388, 0, "Issuer Fundamentals", Never),
    (391, 0, "IBWarrantImpVolCompeteTick", Never),
    (393, 0, "Beta", Never),
    (398, 0, "MarketDataStatus", Never),
    (399, 0, "SShortEx", Never),
    (402, 0, "ADVSchedule", Never),
    (407, 0, "FuturesMargins", Never),
    (411, 0, "rthistvol", Always),
    (418, 0, "Bond Accrued Interest", Never),
    (434, 0, "RFAnalystRating", Never),
    (456, 59, "IBDividends", Types(DIVIDENDS)),
    (459, 0, "RTCLOSE", Never),
    (460, 0, "Bond Factor Multiplier", Always),
    (481, 0, "SpreadScanner", Never),
    (497, 0, "optexiv1", Never),
    (499, 0, "SLB Rate - Fee", Never),
    (504, 0, "DlvPerContractTickTag", Never),
    (511, 0, "hvolrt10 (per-underlying)", Never),
    (512, 0, "hvolrt30 (per-underlying)", Never),
    (513, 0, "hvolrt50 (per-underlying)", Never),
    (514, 0, "hvolrt75 (per-underlying)", Never),
    (515, 0, "hvolrt100 (per-underlying)", Never),
    (516, 0, "hvolrt150 (per-underlying)", Never),
    (517, 0, "hvolrt200 (per-underlying)", Never),
    (530, 0, "DelayedFrozenBidAsk(fzdbidask)", Never),
    (531, 0, "UnderlyingDelayedFrozenMarkPrice(fzdtuplpr)", Never),
    (532, 0, "UnderlyingDelayedFrozenBidAsk(fzduba)", Never),
    (540, 0, "PL_YIELD", Never),
    (548, 0, "SIAnalystRating", Never),
    (561, 0, "Misc. Lag To Date Prices", Never),
    (562, 0, "Misc. Lag Prices", Never),
    (577, 0, "EtfNavLast(navlast)", Always),
    (584, 0, "Average Opening Vol.", Never),
    (585, 0, "Average Closing Vol.", Never),
    (586, 0, "IPOHLMPRC", Always),
    (587, 0, "Pl Price Delayed", Always),
    (588, 0, "Futures Open Interest", Always),
    (595, 0, "Short-Term Volume X Mins", Types(SHORT_TERM_VOLUME)),
    (608, 0, "EMA N", Never),
    (613, 0, "theoprc2", Never),
    (614, 0, "EtfNavMisc(high/low)", Always),
    (619, 0, "Creditman Slow Mark Price", Always),
    (623, 0, "EtfFrozenNavLast(fznavlast)", Always),
    (645, 428, "Monetary Close Price", Never),
    (657, 0, "Average Daily Trading Volume 4 Weak", Never),
    (658, 0, "avgv1min", Never),
    (661, 0, "ivrank", Never),
    (662, 0, "ivpercntl", Never),
    (663, 0, "ivhilo", Never),
    (664, 0, "hvrank", Never),
    (665, 0, "hvpercntl", Never),
    (666, 0, "hvhilo", Never),
    (669, 0, "historical ratios", Never),
    (678, 91, "Reuters 2", Never),
    (688, 0, "mpivcls2", Never),
    (694, 0, "lastivcls", Never),
    (699, 100, "Social Market Analytics", Never),
    (732, 0, "greeks", Never),
    (733, 0, "grksclose", Never),
    (734, 0, "KFVol3", Never),
    (735, 0, "mpmidpiv3", Never),
    (736, 0, "sba_iv3", Never),
    (737, 0, "slastiv3", Never),
    (750, 0, "REFERENCE_PRICE", Never),
    (752, 0, "shariahcl", Never),
    (787, 0, "OddLotBidAskQuotesTickTag(mdbaodd)", Always),
];

/// The older numbers that also name a series, ascending: each resolves to
/// the series' own number and the legality of the entry it is kept under —
/// which for one of them is not the legality of the number it resolves to:
/// the historical volatility the product is asked under stays legal for
/// every type although the per-underlying series registered over its number
/// is legal for none. The entry under 100 is never reached: the number the
/// venue knows the option volume by answers first.
const ALIASES: &[(i32, i32, Legal)] = &[
    (47, 258, Always),
    (59, 456, Types(DIVIDENDS)),
    (91, 678, Never),
    (100, 699, Never),
    (104, 512, Always),
    (220, 221, Always),
    (221, 232, Always),
    (428, 645, Never),
];

/// The type spellings the venue reads, whatever their case. A spelling it
/// reads as none of them — an empty one included — is stated as the empty
/// spelling and takes the series every type takes.
const TYPE_NAMES: [&str; 25] = [
    "STK", "CFD", "OPT", "FOP", "WAR", "IOPT", "FUT", "FWD", "BAG", "CASH", "IND", "BOND",
    "BILL", "FUND", "FIXED", "SLB", "NEWS", "CMDTY", "BSK", "ICU", "ICS", "PHYSS", "CRYPTO",
    "PDC", "EC",
];

/// The type a request's spelling reads as, under the venue's own reading:
/// the canonical spelling for a name it knows — case-blind — the share
/// spelling for the venue's own short one, the combination spelling for the
/// short one of its own, and the empty spelling for a type it reads as
/// none, "any" included.
pub(crate) fn canonical(sec_type: &str) -> &'static str {
    if sec_type.is_empty()
        || sec_type == "*"
        || sec_type.eq_ignore_ascii_case("NONE")
        || sec_type.eq_ignore_ascii_case("ANY")
    {
        return "";
    }
    if sec_type.eq_ignore_ascii_case("CS") {
        return "STK";
    }
    if sec_type.eq_ignore_ascii_case("COMB") {
        return "BAG";
    }
    TYPE_NAMES.iter().copied().find(|n| n.eq_ignore_ascii_case(sec_type)).unwrap_or("")
}

impl Legal {
    fn allows(self, kind: &str, news_capable: bool) -> bool {
        match self {
            Legal::Never => false,
            Legal::Always => true,
            Legal::News => news_capable,
            Legal::Types(set) => set.contains(&kind),
        }
    }
}

/// Trim an entry the way a gateway trims one: everything at or below a
/// space off both ends, and nothing else — a no-break space is part of the
/// entry and fails the number's reading, as it does there.
fn trim(entry: &str) -> &str {
    let bytes = entry.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start] <= b' ' {
        start += 1;
    }
    while end > start && bytes[end - 1] <= b' ' {
        end -= 1;
    }
    &entry[start..end]
}

/// One entry of a list, as a gateway reads it: the token that names no
/// series — a word, a number the venue does not know, a number no series of
/// this type is legal under, a number cut from the suffix its separator
/// promises — reads as none, and one that names a series by an older number
/// reads as the series' own.
fn resolve_one(entry: &str, kind: &str, news_capable: bool) -> Option<i32> {
    let number = match entry.split_once(':') {
        None => entry,
        // The suffix may be any word, and is no part of the reading — but it
        // has to be there: a token ending in the separator with nothing
        // behind it is a list a gateway cannot read whole.
        Some((head, rest)) if rest.split(':').any(|part| !part.is_empty()) => head,
        Some(_) => return None,
    };
    let n: i32 = number.parse().ok()?;
    let (id, legal) = if n == 221 {
        // The credit mark is answered by the pl-price series' older number:
        // a request for 221 resolves to 232, as an alias resolves.
        alias_of(221)?
    } else if let Ok(at) = CATALOGUE.binary_search_by_key(&n, |series| series.0) {
        let series = CATALOGUE[at];
        (series.0, series.3)
    } else {
        alias_of(n)?
    };
    legal.allows(kind, news_capable).then_some(id)
}

/// What an older number resolves to.
fn alias_of(n: i32) -> Option<(i32, Legal)> {
    ALIASES
        .binary_search_by_key(&n, |alias| alias.0)
        .ok()
        .map(|at| (ALIASES[at].1, ALIASES[at].2))
}

/// The series a list resolves to, as a gateway resolves one — each entry to
/// the venue's own number for the series it names, "mdoff" tokens passed
/// over — or `None` where the list does not read whole: a token no series
/// is behind, or tokens only of "mdoff", which resolve to nothing to ask
/// for. An empty list is no list at all and is never read here.
pub(crate) fn resolve(list: &str, sec_type: &str, news_capable: bool) -> Option<Vec<i32>> {
    let kind = canonical(sec_type);
    let mut resolved = Vec::new();
    for token in list.split(',') {
        // The tokenizer a gateway reads with never yields an empty token:
        // separators beside separators are no entry.
        if token.is_empty() {
            continue;
        }
        let entry = trim(token);
        if entry.eq_ignore_ascii_case("mdoff") {
            continue;
        }
        resolved.push(resolve_one(entry, kind, news_capable)?);
    }
    (!resolved.is_empty()).then_some(resolved)
}

/// The refusal a list that does not read whole is refused with: the list as
/// the caller stated it, the type as the venue reads the request's spelling
/// of it, and every series legal for the two of them, ascending, each under
/// its number — with the older number beside it where it answers one — and
/// its name.
pub(crate) fn incorrect_list(list: &str, sec_type: &str, news_capable: bool) -> String {
    let kind = canonical(sec_type);
    let mut legal = String::new();
    for (id, alias, name, l) in CATALOGUE {
        if !l.allows(kind, news_capable) {
            continue;
        }
        if !legal.is_empty() {
            legal.push(',');
        }
        legal.push_str(&id.to_string());
        if *alias != 0 {
            legal.push('/');
            legal.push_str(&alias.to_string());
        }
        legal.push('(');
        legal.push_str(name);
        legal.push(')');
    }
    format!(
        "Incorrect generic tick list of {list}.  Legal ones for ({kind}) are: {legal}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The catalogue is whole against the registry it is read from: 94
    /// series, ascending and unduplicated, and 8 older numbers resolving
    /// into it.
    #[test]
    fn the_catalogue_is_whole_and_in_order() {
        assert_eq!(CATALOGUE.len(), 94, "every series the venue registers");
        assert!(CATALOGUE.windows(2).all(|w| w[0].0 < w[1].0), "ascending, unduplicated");
        assert_eq!(ALIASES.len(), 8);
        assert!(ALIASES.windows(2).all(|w| w[0].0 < w[1].0), "ascending, unduplicated");
        // Every alias resolves to a series the catalogue holds, and states
        // the legality of the entry it is kept under.
        for (alias, id, _) in ALIASES {
            assert!(CATALOGUE.binary_search_by_key(id, |s| s.0).is_ok(), "{alias} names a series");
        }
    }
}
