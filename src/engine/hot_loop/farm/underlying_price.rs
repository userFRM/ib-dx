//! The underlying's price as a gateway's option model holds it.
//!
//! A gateway watches the quote of every modelled option's underlying on its
//! own, on the listing it prefers for the contract: the smart one where the
//! contract has one, else the only listing it has. It marks the underlying
//! from that quote and hands its model the quote's bid, ask, last and close,
//! each narrowed to single precision, with the mark beside them: nothing until
//! the mark first stands, and then each part of the quote as it changes.
//!
//! The model tick states that mark as the underlying's price where the chain
//! parameters state none, and the bid's, ask's and last's ticks are worked
//! from the mark, else the midpoint of the sides, else the last, else the
//! close.

use super::{FarmState, InstrumentId, SharedState};
use crate::engine::context::Context;

/// A figure the quote does not state.
const UNSET: f64 = f64::MAX;

/// A figure that is one: stated, and a finite number.
fn valid(figure: f64) -> bool {
    figure != UNSET && figure.is_finite()
}

/// What a gateway's model holds of an underlying's price: the parts of its
/// quote, each to single precision and not a number where the quote has not
/// stated one that stands, and the mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UnderlyingPrice {
    pub(super) bid: f64,
    pub(super) ask: f64,
    pub(super) last: f64,
    pub(super) close: f64,
    pub(super) mark: f64,
}

impl UnderlyingPrice {
    const NONE: Self =
        Self { bid: f64::NAN, ask: f64::NAN, last: f64::NAN, close: f64::NAN, mark: f64::NAN };

    /// The price the options' bid, ask and last ticks are worked from: the
    /// mark, else the midpoint of the two sides, else the last, else the
    /// close, as it stands.
    pub(super) fn preferred(&self) -> f64 {
        if valid(self.mark) {
            self.mark
        } else if valid(self.bid) && valid(self.ask) {
            0.5 * (self.bid + self.ask)
        } else if valid(self.last) {
            self.last
        } else {
            self.close
        }
    }
}

/// How a gateway reads an underlying's quote for its mark, by the kind of
/// contract it is.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Kind {
    /// An index, marked at its last.
    index: bool,
    /// Marked at the midpoint of its sides where it trades no last to mark
    /// it at.
    midpoint: bool,
    /// A mark outside the sides is held to them, and a quote with no mark
    /// is marked from its sides.
    within_sides: bool,
}

impl Kind {
    /// The kind of the contract of this type, and whether its definition says
    /// it is quoted at its midpoint.
    pub(super) fn of(sec_type: &str, quoted_at_midpoint: bool) -> Self {
        let (index, midpoint, within_sides) = match sec_type {
            "IND" => (true, false, false),
            "FUND" => (false, false, false),
            "CASH" | "CMDTY" | "FIXED" => (false, true, true),
            _ => (false, false, true),
        };
        Self { index, midpoint: midpoint || quoted_at_midpoint, within_sides }
    }
}

/// The underlying's quote as its mark is read from it.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Quote {
    pub(super) bid: Option<f64>,
    pub(super) ask: Option<f64>,
    pub(super) last: Option<f64>,
    pub(super) close: Option<f64>,
    pub(super) bid_size: Option<f64>,
    pub(super) ask_size: Option<f64>,
    pub(super) last_size: Option<f64>,
    /// The quote's state: a side past its limit (2 the bid, 4 the ask) is
    /// not taken.
    pub(super) state: i64,
    /// The close's attributes: the lowest bit says the close does not stand.
    pub(super) close_attributes: i32,
    /// Whether the quote says trading has halted.
    pub(super) halted: bool,
}

/// What else the mark is read from beside the quote.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Beside {
    /// Whether the contract's prices may be negative.
    pub(super) negative: bool,
    /// The mark the venue states for the contract, where something asked
    /// for it.
    pub(super) venue_mark: Option<f64>,
    /// The market price the account's portfolio states for the contract.
    pub(super) position_price: Option<f64>,
    /// Whether the quote's subscription has been acknowledged.
    pub(super) subscribed: bool,
}

impl Quote {
    fn figure(value: Option<f64>) -> f64 {
        value.unwrap_or(UNSET)
    }

    /// A side a gateway takes: a size other than nought, a price, and not
    /// past its limit.
    fn bid_stands(&self) -> bool {
        self.bid_size.is_some_and(|size| size != 0.0)
            && valid(Self::figure(self.bid))
            && self.state & 2 == 0
    }

    fn ask_stands(&self) -> bool {
        self.ask_size.is_some_and(|size| size != 0.0)
            && valid(Self::figure(self.ask))
            && self.state & 4 == 0
    }

    /// A side the midpoint is taken from: a size other than nought, and a
    /// price above nought or one the contract's prices may take.
    fn quoted(size: Option<f64>, price: Option<f64>, negative: bool) -> bool {
        let price = Self::figure(price);
        size.is_some_and(|size| size != 0.0) && valid(price) && (price > 0.0 || negative)
    }

    /// A last a gateway takes: a price, and either a size above nought or a
    /// price above nought, or an index's where its prices may be negative.
    fn last_stands(&self, kind: Kind, negative: bool) -> bool {
        let last = Self::figure(self.last);
        valid(last)
            && (self.last_size.is_some_and(|size| size > 0.0)
                || last > 0.0
                || (kind.index && negative))
    }

    /// A close a gateway takes: a price at or above nought, or one the
    /// contract's prices may take, whose attributes do not say it does not
    /// stand.
    fn close_stands(&self, negative: bool) -> bool {
        let close = Self::figure(self.close);
        close != UNSET
            && (close >= 0.0 || negative)
            && !(self.close_attributes != -1 && self.close_attributes & 1 != 0)
    }
}

/// The underlying's mark as a gateway reads it off the quote.
fn show_quote(quote: &Quote, kind: Kind, beside: &Beside) -> f64 {
    let (bid, ask) = (Quote::figure(quote.bid), Quote::figure(quote.ask));
    let (last, close) = (Quote::figure(quote.last), Quote::figure(quote.close));
    // The mark the venue states, where something asked it for the contract.
    if let Some(mark) = beside.venue_mark.filter(|mark| valid(*mark)) {
        return mark;
    }
    let position = beside.position_price.filter(|price| valid(*price));
    let last_stands = quote.last_stands(kind, beside.negative);
    let bid_quoted = Quote::quoted(quote.bid_size, quote.bid, beside.negative);
    let ask_quoted = Quote::quoted(quote.ask_size, quote.ask, beside.negative);
    // The portfolio's price, where trading has halted or the subscription
    // stands with nothing quoted.
    if (quote.halted || (beside.subscribed && !last_stands && !bid_quoted && !ask_quoted))
        && let Some(price) = position
    {
        return price;
    }
    let mark = if kind.index {
        if last_stands && (last >= 0.0 || beside.negative) { last } else { UNSET }
    } else if last_stands {
        last
    } else if kind.midpoint && bid_quoted && ask_quoted {
        (bid + ask) / 2.0
    } else if quote.close_stands(beside.negative) {
        close
    } else {
        UNSET
    };
    let (bid_stands, ask_stands) = (quote.bid_stands(), quote.ask_stands());
    if valid(mark) {
        if !kind.within_sides {
            return mark;
        }
        let crossed = bid_stands && ask_stands && bid > ask;
        return if bid_stands && mark < bid && !crossed {
            bid
        } else if ask_stands && mark > ask && !crossed {
            ask
        } else {
            mark
        };
    }
    if kind.within_sides {
        if ask_stands {
            return ((if bid_stands { bid } else { 0.0 }) + ask) / 2.0;
        }
        if bid_stands {
            return bid;
        }
    }
    position.unwrap_or(UNSET)
}

/// What the watch last read of the quote, to tell what moved.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Seen {
    bid: Option<f64>,
    ask: Option<f64>,
    last: Option<f64>,
    close: Option<f64>,
    bid_size: Option<f64>,
    ask_size: Option<f64>,
}

/// Take in a reading of the underlying's quote as a gateway's watch takes it
/// in, and say whether what the model holds of the price moved.
///
/// The bid and ask are taken where they are quoted, the last and the close
/// where they stand. The mark is the quote's, or the last where the quote's
/// mark does not stand. Each is multiplied by `scale`, where the underlying is
/// priced off another contract's quote. Nothing is handed on until the mark
/// first stands; then all of it at once, each part to single precision, and
/// afterwards the close when the close moved, the last when the last moved,
/// and the two sides when either moved or its size went to nought, each with
/// the mark as it stands.
pub(super) fn take_in(
    held: &mut Option<UnderlyingPrice>,
    seen: &mut Seen,
    quote: &Quote,
    kind: Kind,
    beside: &Beside,
    scale: f64,
) -> bool {
    let negative = beside.negative;
    let nonnegative = |figure: f64| valid(figure) && (negative || figure >= 0.0);
    let taken = |figure: Option<f64>, stands: bool| {
        if stands { Quote::figure(figure) * scale } else { f64::NAN }
    };
    let bid = taken(quote.bid, quote.bid_stands());
    let ask = taken(quote.ask, quote.ask_stands());
    let close = taken(quote.close, quote.close_stands(negative));
    let last = taken(quote.last, quote.last_stands(kind, negative));
    let mut mark = show_quote(quote, kind, beside);
    if !nonnegative(mark) {
        mark = Quote::figure(quote.last);
        if !quote.last_stands(kind, negative) {
            mark = f64::NAN;
        }
    }
    let mark = mark * scale;
    let now = Seen {
        bid: quote.bid,
        ask: quote.ask,
        last: quote.last,
        close: quote.close,
        bid_size: quote.bid_size,
        ask_size: quote.ask_size,
    };
    let was = std::mem::replace(seen, now);
    let size_went = |before: Option<f64>, after: Option<f64>| before != after && after == Some(0.0);
    let sides_moved = was.bid != now.bid
        || was.ask != now.ask
        || size_went(was.bid_size, now.bid_size)
        || size_went(was.ask_size, now.ask_size);
    let first = held.is_none();
    if first && !nonnegative(mark) {
        return false;
    }
    let before = *held;
    let price = held.get_or_insert(UnderlyingPrice::NONE);
    let narrowed = |figure: f64| nonnegative(figure).then_some(figure as f32 as f64);
    if first || was.close != now.close {
        price.close = narrowed(close).unwrap_or(f64::NAN);
        price.mark = mark;
    }
    if first || was.last != now.last {
        price.last = narrowed(last).unwrap_or(f64::NAN);
        price.mark = mark;
    }
    if first || sides_moved {
        let (bid, ask) = (narrowed(bid), narrowed(ask));
        if bid.is_some() || ask.is_some() {
            price.bid = bid.unwrap_or(price.bid);
            price.ask = ask.unwrap_or(price.ask);
            price.mark = mark;
        }
    }
    *held != before
}

impl FarmState {
    /// Take in what the quote a watch is served on now states, for each
    /// underlying watched there, and put the options on it up for their ticks
    /// again where the price the model holds moved.
    ///
    /// The quote is read as the watched contract's own: by its type, its
    /// prices' rule, its definition, its mark and its holding.
    pub(super) fn note_underlying_quote(
        &mut self,
        instrument: InstrumentId,
        context: &Context,
        shared: &SharedState,
    ) {
        for at in 0..self.underlying_models.len() {
            if self.underlying_models[at].watch != Some(instrument) {
                continue;
            }
            let watched = self.underlying_models[at].watched;
            let definition = u32::try_from(watched)
                .ok()
                .and_then(|con_id| shared.reference.contract_definition(con_id, ""));
            let quoted_at_midpoint = definition.as_ref().is_some_and(|definition| {
                definition
                    .order_type_rules
                    .iter()
                    .any(|(name, code)| name == "USEMID" && *code != 4)
            });
            let negative = definition
                .as_ref()
                .and_then(|definition| definition.market_rule_id)
                .and_then(|rule| shared.reference.market_rule(rule as i32))
                .is_some_and(|rule| rule.negative_prices);
            let sec_type = match &definition {
                Some(definition) => definition.sec_type.to_api_str().to_string(),
                None => self.underlying_models[at].under_sec_type.clone(),
            };
            let views = shared.market.pricing_quote_views(instrument);
            let record = views.records[views.mode.clamp(0, 3) as usize];
            let price = |value: Option<crate::types::Price>| {
                value.map(|value| value as f64 / crate::types::PRICE_SCALE as f64)
            };
            let size = |value: Option<crate::types::Qty>| {
                value.map(|value| value as f64 / crate::types::QTY_SCALE as f64)
            };
            let quote = Quote {
                bid: price(record.bid),
                ask: price(record.ask),
                last: price(record.last),
                close: price(record.close),
                bid_size: size(record.bid_size),
                ask_size: size(record.ask_size),
                last_size: size(record.last_size),
                state: record.state_mask,
                close_attributes: record.close_attributes,
                halted: self.quote_flags.get(&instrument).is_some_and(|flags| flags & 1 != 0),
            };
            // The venue's mark for the contract, which a gateway holds for
            // every listing of it whichever one asked for it.
            let venue_mark = views.mark_price.or_else(|| {
                context
                    .market
                    .active_instruments()
                    .filter(|(other, held)| *other != instrument && *held == watched)
                    .find_map(|(other, _)| shared.market.pricing_quote_views(other).mark_price)
            });
            let beside = Beside {
                negative,
                venue_mark,
                position_price: shared
                    .portfolio
                    .position_info(watched)
                    .filter(|held| held.market_price_stated)
                    .map(|held| held.market_price as f64 / crate::types::PRICE_SCALE as f64),
                subscribed: views.confirmed,
            };
            let kind = Kind::of(&sec_type, quoted_at_midpoint);
            let held = &mut self.underlying_models[at];
            if !take_in(&mut held.price, &mut held.seen, &quote, kind, &beside, held.scale) {
                continue;
            }
            for option in held.options.clone() {
                self.option_ticks_due.insert(option);
                if let Some(modelled) = self.modelled_options.get_mut(&option) {
                    modelled.sides_due = true;
                }
            }
        }
    }

    /// Whether the underlyings a watch on a contract would serve are priced
    /// off another contract's quote, and so watched there instead, as a
    /// gateway watches them: an index whose definition names the contract it
    /// is priced off (6577), where it names no other way of pricing it (8127,
    /// 8148), with each of that contract's prices multiplied by the figure it
    /// states beside it (6578). Decided once the venue has named the index,
    /// before its own quote is asked for.
    pub(crate) fn priced_off_another(&mut self, con_id: i64, shared: &SharedState) -> bool {
        let definition = u32::try_from(con_id)
            .ok()
            .and_then(|con_id| shared.reference.contract_definition(con_id, ""));
        let stated = |tag: u32| {
            definition.as_ref().and_then(|definition| {
                definition.unnamed_fields.iter().find(|(at, _)| *at == tag).map(|(_, v)| v.clone())
            })
        };
        let Some(other) = stated(6577)
            .and_then(|named| named.parse::<i64>().ok())
            .filter(|named| *named > 0 && *named != con_id)
            .filter(|_| stated(8148).is_none() && stated(8127).is_none())
        else {
            return false;
        };
        let scale = stated(6578).and_then(|scale| scale.parse().ok()).unwrap_or(0.0);
        let mut moved = false;
        for held in self.underlying_models.iter_mut().filter(|held| {
            held.watched == con_id && held.con_id == con_id && held.under_sec_type == "IND"
        }) {
            held.watched = other;
            held.scale = scale;
            self.watches_ended.push(con_id);
            self.watches_wanted.push((other, String::new()));
            moved = true;
        }
        moved
    }

    /// Note the slot a watch is served on, once the venue has named the
    /// contract watched and the watch is taken.
    pub(crate) fn note_underlying_watch(&mut self, con_id: i64, instrument: InstrumentId) {
        for held in self.underlying_models.iter_mut().filter(|held| held.watched == con_id) {
            held.watch = Some(instrument);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A share's quote, stated whole.
    fn share(bid: f64, ask: f64, last: f64, close: f64) -> Quote {
        Quote {
            bid: Some(bid),
            ask: Some(ask),
            last: Some(last),
            close: Some(close),
            bid_size: Some(1.0),
            ask_size: Some(1.0),
            last_size: Some(1.0),
            close_attributes: -1,
            ..Quote::default()
        }
    }

    /// The mark as a gateway reads it off a quote, row by row, and each
    /// figure as the rule a gateway applies works it.
    #[test]
    fn the_underlying_is_marked_as_a_gateway_marks_it() {
        let stock = Kind::of("STK", false);
        let index = Kind::of("IND", false);
        let pair = Kind::of("CASH", false);
        let open = Beside { subscribed: true, ..Beside::default() };
        let held = Beside { position_price: Some(99.0), ..open };
        let halted = Quote { halted: true, ..share(10.0, 10.5, 10.25, 9.0) };
        let past_limit = Quote { state: 2, ..share(10.0, 10.5, 9.5, 9.0) };
        let closed = Quote { last: None, last_size: None, ..share(10.0, 10.5, 0.0, 9.5) };
        let rows: [(&str, Quote, Kind, Beside, f64); 12] = [
            ("the last, within the sides", share(10.0, 10.5, 10.25, 9.0), stock, open, 10.25),
            ("a last under the bid is the bid", share(10.0, 10.5, 9.5, 9.0), stock, open, 10.0),
            ("a last over the ask is the ask", share(10.0, 10.5, 11.0, 9.0), stock, open, 10.5),
            ("crossed sides hold nothing", share(10.5, 10.0, 11.0, 9.0), stock, open, 11.0),
            ("a bid past its limit holds nothing", past_limit, stock, open, 9.5),
            ("the close where no last stands, held to the sides", closed, stock, open, 10.0),
            (
                "the venue's own mark first",
                share(10.0, 10.5, 10.25, 9.0),
                stock,
                Beside { venue_mark: Some(10.3), ..open },
                10.3,
            ),
            ("the portfolio's price while halted", halted, stock, held, 99.0),
            (
                "the portfolio's price with nothing quoted",
                Quote { close_attributes: -1, ..Quote::default() },
                stock,
                held,
                99.0,
            ),
            ("an index at its last, unheld", share(10.0, 10.5, 11.0, 9.0), index, open, 11.0),
            (
                "a pair at the midpoint of its sides",
                Quote { last: None, ..share(1.25, 1.5, 0.0, 1.0) },
                pair,
                open,
                1.375,
            ),
            (
                "the sides where nothing else stands",
                Quote { last: None, close: None, ..share(10.0, 10.5, 0.0, 0.0) },
                stock,
                open,
                10.25,
            ),
        ];
        for (named, quote, kind, beside, expected) in rows {
            assert_eq!(show_quote(&quote, kind, &beside), expected, "{named}");
        }
    }

    /// Nothing is handed on until the mark first stands, then all of it; and
    /// afterwards only the part that moved, with the mark as it stands then.
    #[test]
    fn the_price_is_handed_on_from_the_first_mark_and_then_as_it_moves() {
        let stock = Kind::of("STK", false);
        let beside = Beside { subscribed: true, ..Beside::default() };
        let (mut held, mut seen) = (None, Seen::default());
        let quiet = Quote { close: Some(9.0), close_attributes: 1, ..Quote::default() };
        assert!(!take_in(&mut held, &mut seen, &quiet, stock, &beside, 1.0));
        assert_eq!(held, None, "no mark stands, nothing is held");

        let first = share(10.1, 10.5, 10.3, 9.0);
        assert!(take_in(&mut held, &mut seen, &first, stock, &beside, 1.0));
        assert_eq!(
            held,
            Some(UnderlyingPrice {
                bid: 10.1f32 as f64,
                ask: 10.5,
                last: 10.3f32 as f64,
                close: 9.0,
                mark: 10.3,
            }),
        );
        // The bid moves over the last: the mark is held to it, and handed on
        // with the sides; the last the model holds is the one it took.
        let bid_over = Quote { bid: Some(10.4), ..first };
        assert!(take_in(&mut held, &mut seen, &bid_over, stock, &beside, 1.0));
        assert_eq!(
            held.map(|price| (price.bid, price.last, price.mark, price.preferred())),
            Some((10.4f32 as f64, 10.3f32 as f64, 10.4, 10.4)),
        );
        // Only the last's size moves: nothing is handed on.
        let sized = Quote { last_size: Some(3.0), ..bid_over };
        assert!(!take_in(&mut held, &mut seen, &sized, stock, &beside, 1.0));

        // Priced off another contract's quote, every price is multiplied by
        // the figure stated for it.
        let (mut held, mut seen) = (None, Seen::default());
        assert!(take_in(&mut held, &mut seen, &first, stock, &beside, 10.0));
        assert_eq!(
            held,
            Some(UnderlyingPrice { bid: 101.0, ask: 105.0, last: 103.0, close: 90.0, mark: 103.0 }),
        );
    }
}
