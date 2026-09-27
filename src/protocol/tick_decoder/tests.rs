//! The tests for this module.
//!
//! One file per module, as `api/client` already does it. Each block below
//! reaches the code it tests through `super::super`, which is the module this
//! file belongs to.

use super::*;

#[test]
fn bit_reader_basic() {
    let data = [0b1010_0011, 0b1100_0000];
    let mut r = BitReader::new(&data, 16);
    assert_eq!(r.read_unsigned(4), Some(0b1010)); // 10
    assert_eq!(r.read_unsigned(4), Some(0b0011)); // 3
    assert_eq!(r.read_unsigned(2), Some(0b11));   // 3
    assert_eq!(r.remaining(), 6);
}

#[test]
fn bit_reader_single_bits() {
    let data = [0b10110000];
    let mut r = BitReader::new(&data, 5);
    assert_eq!(r.read_unsigned(1), Some(1));
    assert_eq!(r.read_unsigned(1), Some(0));
    assert_eq!(r.read_unsigned(1), Some(1));
    assert_eq!(r.read_unsigned(1), Some(1));
    assert_eq!(r.read_unsigned(1), Some(0));
    assert_eq!(r.read_unsigned(1), None); // exhausted
}

#[test]
fn bit_reader_overflow() {
    let data = [0xFF];
    let mut r = BitReader::new(&data, 8);
    assert_eq!(r.read_unsigned(9), None); // not enough bits
}

#[test]
fn bit_reader_total_bits_capped_to_data_length() {
    // total_bits exceeds data.len() * 8 — must be capped, not panic
    let data = [0xFF, 0xAA]; // 16 bits of data
    let mut r = BitReader::new(&data, 1000); // claim 1000 bits
    assert_eq!(r.remaining(), 16); // capped to 16
    assert_eq!(r.read_unsigned(8), Some(0xFF));
    assert_eq!(r.read_unsigned(8), Some(0xAA));
    assert_eq!(r.read_unsigned(1), None); // no more data
}

#[test]
fn bit_reader_empty_data_with_nonzero_bits() {
    let data: [u8; 0] = [];
    let r = BitReader::new(&data, 100);
    assert_eq!(r.remaining(), 0);
}

#[test]
fn decode_ticks_empty() {
    assert!(decode_ticks_35p(&[]).is_empty());
    assert!(decode_ticks_35p(&[0, 0]).is_empty());
}

// ── Helper: build bit-packed payloads for decode_ticks_35p ──────────

/// Accumulates individual bits (MSB-first order) and produces the
/// complete 35=P body: 2-byte big-endian bit_count + payload bytes.
pub(super) struct PayloadBuilder {
    bits: Vec<u8>, // each element is 0 or 1
}

impl PayloadBuilder {
    pub(super) fn new() -> Self {
        Self { bits: Vec::new() }
    }

    /// Push `n` bits from the MSB side of `val`. Widths above 64 are
    /// zero-filled on the left, so a field wider than a `u64` can be built
    /// — which is the point of the extended entries this exercises.
    fn push(&mut self, val: u64, n: usize) {
        for _ in 64..n {
            self.bits.push(0);
        }
        for i in (0..n.min(64)).rev() {
            self.bits.push(((val >> i) & 1) as u8);
        }
    }

    /// Emit a server-tag header: 1-bit continuation + 31-bit tag.
    pub(super) fn server_tag(&mut self, cont: u64, tag: u32) {
        self.push(cont, 1);
        self.push(tag as u64, 31);
    }

    /// Emit a normal tick entry.
    /// `has_more`: 0 or 1.
    /// `width_bytes`: 1..=4 (maps to raw_width 0..=3).
    /// `value`: absolute value written into `width_bytes * 8 - 1` bits.
    /// `negative`: if true the sign bit is 1.
    pub(super) fn tick(&mut self, tick_type: u64, has_more: u64, width_bytes: u64, value: u64, negative: bool) {
        assert!(tick_type < 31);
        assert!((1..=4).contains(&width_bytes));
        self.push(tick_type, 5);
        self.push(has_more, 1);
        self.push(width_bytes - 1, 2); // raw_width
        // sign bit + magnitude
        let total_value_bits = (width_bytes * 8) as usize;
        self.push(if negative { 1 } else { 0 }, 1);
        self.push(value, total_value_bits - 1);
    }

    /// Emit an extended tick entry (raw_tick_type == 31).
    pub(super) fn tick_extended(
        &mut self,
        has_more: u64,
        ext_tick_type: u64,
        ext_byte_width: u64,
        value: u64,
        negative: bool,
    ) {
        self.tick_shifted(has_more, ext_tick_type, 0, ext_byte_width, value, negative);
    }

    /// The same, stating how far the decimal point moves.
    pub(super) fn tick_shifted(
        &mut self,
        has_more: u64,
        ext_tick_type: u64,
        decimal_shift: u64,
        ext_byte_width: u64,
        value: u64,
        negative: bool,
    ) {
        self.push(31, 5); // sentinel
        self.push(has_more, 1);
        self.push(0, 2); // raw_width (ignored for extended)
        self.push(ext_tick_type, 8);
        self.push(decimal_shift, 4);
        self.push(ext_byte_width, 4);
        let total_value_bits = (ext_byte_width * 8) as usize;
        self.push(if negative { 1 } else { 0 }, 1);
        self.push(value, total_value_bits - 1);
    }

    /// Finalize into the full body: [bit_count_hi, bit_count_lo, payload…]
    pub(super) fn build(&self) -> Vec<u8> {
        let bit_count = self.bits.len();
        let byte_count = bit_count.div_ceil(8);
        let mut payload = vec![0u8; byte_count];
        for (i, &b) in self.bits.iter().enumerate() {
            if b == 1 {
                payload[i >> 3] |= 1 << (7 - (i & 7));
            }
        }
        let mut body = Vec::with_capacity(2 + byte_count);
        body.push((bit_count >> 8) as u8);
        body.push((bit_count & 0xFF) as u8);
        body.extend_from_slice(&payload);
        body
    }
}

/// The width comes off the wire, so the step has to be bounded by what is
/// actually left rather than trusted. An unchecked addition wraps in
/// release and rewinds the reader instead of stopping it.
#[test]
fn skipping_past_the_end_refuses_rather_than_wrapping() {
    let mut reader = BitReader::new(&[0xFF, 0xFF], 16);
    assert!(reader.read_unsigned(1).is_some());
    assert!(!reader.skip(usize::MAX), "an absurd width is refused");
    assert!(!reader.skip(16), "and so is one just past the end");
    assert_eq!(reader.remaining(), 15, "the position is untouched by either");
        assert!(reader.skip(15), "while a width that fits still advances");
    assert_eq!(reader.remaining(), 0);
}

/// An extended entry states its width in a full byte, so it can name a
/// value wider than this decoder reads. That entry is lost either way;
/// abandoning the message would also discard every tick after it, including
/// the other server tags in the same 35=P, so a quote update sitting behind
/// one would never arrive.
#[test]
fn a_tick_too_wide_to_read_does_not_discard_the_rest_of_the_message() {
    let mut b = PayloadBuilder::new();
    b.server_tag(1, 100);
    // Nine bytes: 72 value bits, which the reader cannot return.
    b.tick_extended(1, 40, 9, 7, false);
    b.tick(2, 0, 4, 12_345, false);
    b.server_tag(0, 200);
    b.tick(3, 0, 4, 678, false);

    let ticks = decode_ticks_35p(&b.build());

    let seen: Vec<(u32, u64, i64)> = ticks.iter()
        .map(|t| (t.server_tag, t.tick_type, t.magnitude))
        .collect();
    assert!(
        seen.contains(&(100, 2, 12_345)),
        "the tick after the wide one, under the same server tag: {seen:?}",
    );
    assert!(
        seen.contains(&(200, 3, 678)),
        "and the whole server tag after it: {seen:?}",
    );
    assert!(
        !seen.iter().any(|(_, ty, _)| *ty == 40),
        "the entry itself is still dropped, not guessed at: {seen:?}",
    );
}

/// A frame stating more bits than it holds is a frame cut short, and is
/// refused rather than read as the shorter frame it is not: read short, part
/// of the ticks would arrive as the whole of them.
#[test]
fn a_frame_cut_short_by_its_own_length_is_refused() {
    let mut b = PayloadBuilder::new();
    b.server_tag(1, 9);
    b.tick(2, 0, 2, 501, false);
    b.server_tag(0, 10);
    b.tick(3, 0, 2, 502, false);
    let mut body = b.build();
    assert_eq!(decode_ticks_35p(&body).len(), 2, "the whole frame reads");

    // Cut so the first tick is whole and the second is not. Read short, the
    // first tick would pass as the frame's whole content.
    body.truncate(body.len() - 3);
    assert!(
        decode_ticks_35p(&body).is_empty(),
        "the cut one is refused, not shortened",
    );
}

/// The count of bits is two bytes, so it wraps at sixty-five thousand five
/// hundred and thirty-six. What was carried is recovered against how much
/// arrived — read as stated, a long batch decodes as its own first eight
/// kilobytes and the records behind that are lost, and one exactly a cycle
/// long, which states nought, is lost whole.
#[test]
fn a_batch_past_the_wrap_of_its_own_bit_count_is_read_whole() {
    // Each of these is seventy-two bits: a header and one four-byte value.
    let filler = |b: &mut PayloadBuilder, n: usize| {
        for i in 0..n {
            b.server_tag(0, 1_000 + i as u32);
            b.tick(2, 0, 4, 500 + i as u64, false);
        }
    };

    let mut b = PayloadBuilder::new();
    filler(&mut b, 914);
    b.server_tag(0, 7);
    b.tick(3, 0, 4, 4_242, false);
    let body = b.build();
    assert_eq!(
        u16::from_be_bytes([body[0], body[1]]), 344,
        "the count has wrapped once, which is what this exercises",
    );
    let ticks = decode_ticks_35p(&body);
    assert!(
        ticks.iter().any(|t| t.server_tag == 1_000 && t.magnitude == 500),
        "the first record reads",
    );
    assert!(
        ticks.iter().any(|t| t.server_tag == 7 && t.magnitude == 4_242),
        "and so does the one past the wrap: {} records read", ticks.len(),
    );

    // And the boundary itself: nine hundred and nine of those, then a record
    // of eighty-eight bits, is sixty-five thousand five hundred and
    // thirty-six bits exactly — a full cycle, stated as nought.
    let mut b = PayloadBuilder::new();
    filler(&mut b, 909);
    b.server_tag(0, 11);
    b.tick(4, 1, 4, 1_234, false);
    b.tick(5, 0, 1, 99, false);
    let body = b.build();
    assert_eq!(
        u16::from_be_bytes([body[0], body[1]]), 0,
        "a cycle exactly states nought",
    );
    let ticks = decode_ticks_35p(&body);
    assert!(
        ticks.iter().any(|t| t.server_tag == 11 && t.magnitude == 99),
        "the last record of a batch a cycle long reads: {} records read", ticks.len(),
    );
}

// ── decode_ticks_35p tests ──────────────────────────────────────────

#[test]
fn decode_single_tag_single_bid_size_tick() {
    // O_BID_SIZE = 4, width 1 byte, unsigned value 42, positive
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 7); // cont=0, tag=7
    b.tick(O_BID_SIZE, 0, 1, 42, false); // has_more=0
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].server_tag, 7);
    assert_eq!(ticks[0].tick_type, O_BID_SIZE);
    assert_eq!(ticks[0].magnitude, 42);
}

#[test]
fn decode_single_tag_single_bid_price_signed() {
    // O_BID_PRICE = 0, width 2 bytes, value 500, negative (signed delta)
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 100);
    b.tick(O_BID_PRICE, 0, 2, 500, true);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].server_tag, 100);
    assert_eq!(ticks[0].tick_type, O_BID_PRICE);
    assert_eq!(ticks[0].magnitude, -500);
}

#[test]
fn decode_multiple_ticks_for_one_server_tag() {
    // Two ticks under the same server_tag via has_more=1 on first tick
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 55);
    b.tick(O_BID_PRICE, 1, 1, 10, false); // has_more=1
    b.tick(O_ASK_PRICE, 0, 1, 20, false); // has_more=0
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].server_tag, 55);
    assert_eq!(ticks[0].tick_type, O_BID_PRICE);
    assert_eq!(ticks[0].magnitude, 10);
    assert_eq!(ticks[1].server_tag, 55);
    assert_eq!(ticks[1].tick_type, O_ASK_PRICE);
    assert_eq!(ticks[1].magnitude, 20);
}

#[test]
fn decode_multiple_server_tags() {
    // Two server tags, one tick each
    let mut b = PayloadBuilder::new();
    b.server_tag(1, 10); // cont=1 (continuation)
    b.tick(O_VOLUME, 0, 2, 9999, false);
    b.server_tag(0, 20); // cont=0
    b.tick(O_LAST_PRICE, 0, 1, 3, true);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].server_tag, 10);
    assert_eq!(ticks[0].tick_type, O_VOLUME);
    assert_eq!(ticks[0].magnitude, 9999);
    assert_eq!(ticks[1].server_tag, 20);
    assert_eq!(ticks[1].tick_type, O_LAST_PRICE);
    assert_eq!(ticks[1].magnitude, -3);
}

#[test]
fn decode_width_1_byte() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 1);
    b.tick(O_BID_SIZE, 0, 1, 127, false); // max 7-bit unsigned = 127
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, 127);
}

#[test]
fn decode_width_2_bytes() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 2);
    // 2-byte value: 15 bits of magnitude, max 32767
    b.tick(O_ASK_SIZE, 0, 2, 32767, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, 32767);
}

#[test]
fn decode_width_3_bytes() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 3);
    // 3-byte value: 23 bits of magnitude
    b.tick(O_LAST_SIZE, 0, 3, 1_000_000, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, 1_000_000);
}

#[test]
fn decode_width_4_bytes() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 4);
    // 4-byte value: 31 bits of magnitude
    let big_val = 2_000_000_000u64;
    b.tick(O_VOLUME, 0, 4, big_val, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, big_val as i64);
}

#[test]
fn decode_negative_magnitude() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 9);
    b.tick(O_HIGH_PRICE, 0, 2, 1234, true); // negative
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, -1234);
}

#[test]
fn decode_extended_tick_type() {
    // raw_tick_type == 31 triggers extended: 8-bit tick_type + 8-bit byte_width
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 42);
    // Extended tick carrying O_CLOSE_PRICE, byte_width=2, value=777, positive
    b.tick_extended(0, O_CLOSE_PRICE, 2, 777, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].server_tag, 42);
    assert_eq!(ticks[0].tick_type, O_CLOSE_PRICE);
    assert_eq!(ticks[0].magnitude, 777);
}

#[test]
fn decode_extended_tick_type_negative() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 50);
    b.tick_extended(0, O_LAST_TS, 3, 12345, true);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].tick_type, O_LAST_TS);
    assert_eq!(ticks[0].magnitude, -12345);
}

#[test]
fn decode_zero_bit_count() {
    // A payload long enough that the decoder's own loop would run on it: the
    // loop needs more than thirty-two bits, so a two-byte payload passed this
    // whatever the count meant, and the case it was written for went untested.
    let mut body = vec![0u8, 0]; // bit_count = 0
    body.extend(std::iter::repeat_n(0xAAu8, 64));
    let ticks = decode_ticks_35p(&body);
    assert!(
        ticks.is_empty(),
        "a frame stating no bits carries no ticks, whatever follows the count: {} read",
        ticks.len(),
    );
}

#[test]
fn decode_insufficient_bits_for_server_tag() {
    // bit_count = 16 (only 16 bits), not enough for 32-bit server_tag header
    let body = [0u8, 16, 0xFF, 0xFF];
    let ticks = decode_ticks_35p(&body);
    assert!(ticks.is_empty());
}

#[test]
fn decode_insufficient_bits_for_tick_value() {
    // Server tag fits but tick value is truncated
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 1);
    // Start writing tick header but only 5+1+2 = 8 bits of header,
    // then width=4 means 32 bits needed for value, which won't exist.
    b.push(O_BID_PRICE, 5);
    b.push(0, 1); // has_more=0
    b.push(3, 2); // raw_width=3 → byte_width=4 → needs 32 value bits
    // Only provide 8 bits of value instead of 32
    b.push(0xFF, 8);
    let ticks = decode_ticks_35p(&b.build());
    // Should return empty: server_tag read ok, but tick value bits insufficient
    assert!(ticks.is_empty());
}

#[test]
fn decode_extended_insufficient_bits() {
    // Extended tick where the 8+8 extension bits are not fully available
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 1);
    b.push(31, 5); // raw_tick_type = 31 (extended)
    b.push(0, 1);  // has_more
    b.push(0, 2);  // raw_width (ignored)
    // Need 16 more bits for ext_tick_type + ext_byte_width, only provide 4
    b.push(0, 4);
    let ticks = decode_ticks_35p(&b.build());
    assert!(ticks.is_empty());
}

#[test]
fn decode_magnitude_zero() {
    // Value 0 with sign=0 → magnitude 0
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 1);
    b.tick(O_BID_PRICE, 0, 1, 0, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, 0);
}

#[test]
fn decode_negative_zero() {
    // Value 0 with sign=1 → magnitude -0 = 0
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 1);
    b.tick(O_BID_PRICE, 0, 1, 0, true);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].magnitude, 0); // -0 == 0 in i64
}

#[test]
fn decode_three_ticks_chained() {
    // Three ticks under one server_tag: has_more=1, has_more=1, has_more=0
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 999);
    b.tick(O_BID_PRICE, 1, 1, 5, false);
    b.tick(O_ASK_PRICE, 1, 1, 10, true);
    b.tick(O_LAST_PRICE, 0, 2, 300, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 3);
    assert_eq!(ticks[0].tick_type, O_BID_PRICE);
    assert_eq!(ticks[0].magnitude, 5);
    assert_eq!(ticks[1].tick_type, O_ASK_PRICE);
    assert_eq!(ticks[1].magnitude, -10);
    assert_eq!(ticks[2].tick_type, O_LAST_PRICE);
    assert_eq!(ticks[2].magnitude, 300);
    for t in &ticks {
        assert_eq!(t.server_tag, 999);
    }
}

#[test]
fn decode_body_too_short() {
    // body < 4 bytes triggers early return
    assert!(decode_ticks_35p(&[0]).is_empty());
    assert!(decode_ticks_35p(&[0, 10]).is_empty());
    assert!(decode_ticks_35p(&[0, 10, 0xFF]).is_empty());
}

#[test]
fn decode_mixed_server_tags_and_ticks() {
    // Tag1 with 2 ticks, then Tag2 with 1 tick
    let mut b = PayloadBuilder::new();
    b.server_tag(1, 111);
    b.tick(O_BID_SIZE, 1, 1, 50, false);
    b.tick(O_ASK_SIZE, 0, 1, 60, false);
    b.server_tag(0, 222);
    b.tick(O_LAST_SIZE, 0, 2, 1000, true);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 3);
    assert_eq!(ticks[0].server_tag, 111);
    assert_eq!(ticks[0].tick_type, O_BID_SIZE);
    assert_eq!(ticks[0].magnitude, 50);
    assert_eq!(ticks[1].server_tag, 111);
    assert_eq!(ticks[1].tick_type, O_ASK_SIZE);
    assert_eq!(ticks[1].magnitude, 60);
    assert_eq!(ticks[2].server_tag, 222);
    assert_eq!(ticks[2].tick_type, O_LAST_SIZE);
    assert_eq!(ticks[2].magnitude, -1000);
}

#[test]
fn decode_extended_with_has_more() {
    // Extended tick with has_more=1, followed by a normal tick
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 77);
    b.tick_extended(1, O_LAYOUT, 1, 1, false); // has_more=1
    b.tick(O_BID_PRICE, 0, 1, 99, false);      // has_more=0
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].tick_type, O_LAYOUT);
    assert_eq!(ticks[0].magnitude, 1);
    assert_eq!(ticks[1].tick_type, O_BID_PRICE);
    assert_eq!(ticks[1].magnitude, 99);
}

#[test]
fn decode_max_server_tag() {
    // Maximum 31-bit server_tag value
    let max_tag = (1u32 << 31) - 1;
    let mut b = PayloadBuilder::new();
    b.server_tag(0, max_tag);
    b.tick(O_BID_SIZE, 0, 1, 1, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].server_tag, max_tag);
}

mod wire_identity_tests {
    use super::super::*;

    /// The `35=P` field identities were wrong for five fields and no test
    /// noticed, because nothing pinned a wire number to a meaning. These are
    /// the numbers measured on a live front-month future; they are
    /// asserted directly so a remap has to change a test that says why.
    #[test]
    fn wire_tick_identities_match_what_the_feed_sends() {
        // Volume is the monotonic one: 228 samples, zero decreases.
        assert_eq!(O_VOLUME, 10);
        // Per-trade size is the one that oscillates: 120 samples, 60 decreases.
        assert_eq!(O_LAST_SIZE, 6);
        // The high read above every last price; the field previously mapped to
        // it read below the last trade, which a high cannot do.
        assert_eq!(O_HIGH_PRICE, 8);
        assert_eq!(O_LOW_PRICE, 9);
        // The timestamp is carried on 20/21, not on the volume field.
        assert_eq!(O_TS_BASE, 20);
        assert_eq!(O_TS_OFFSET, 21);
        // Open and close, settled against the daily bars for the same session:
        // the wire's 22 matched the bar open exactly, and its 3 matched the
        // PRIOR session's close — not the current one. Getting these the wrong
        // way round silently swaps them for every caller, and P&L reads close.
        assert_eq!(O_OPEN_PRICE, 22);
        assert_eq!(O_CLOSE_PRICE, 3);
        // Unchanged and confirmed by the same capture.
        assert_eq!(O_BID_PRICE, 0);
        assert_eq!(O_ASK_PRICE, 1);
        assert_eq!(O_LAST_PRICE, 2);
        assert_eq!(O_BID_SIZE, 4);
        assert_eq!(O_ASK_SIZE, 5);
        // No two fields may share a wire number.
        // Every wire number the decoder names, including the ones with no
        // consumer — a remap colliding with one of those would otherwise pass.
        let all = [
            O_BID_PRICE, O_ASK_PRICE, O_LAST_PRICE, O_BID_SIZE, O_ASK_SIZE,
            O_LAST_SIZE, O_HIGH_PRICE, O_LOW_PRICE, O_VOLUME, O_TS_BASE,
            O_TS_OFFSET, O_OPEN_PRICE, O_CLOSE_PRICE,
            O_LAST_EXCH, O_BID_EXCH, O_ASK_EXCH, O_LAYOUT, O_LAST_TS, O_ELIGIBLE,
            O_QUOTE_STATE,
        ];
        let mut seen = all.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), all.len(), "two fields are mapped to the same wire type");
    }
}
mod overflow_tests {
    use super::super::*;
    use super::PayloadBuilder;

    /// The extended 35=P format decodes a full 8-byte magnitude, which the
    /// quote path then scales. Establish that the decoder really can produce a
    /// value the multiply cannot represent — the apply path drops those rather
    /// than publishing a pinned or wrapped price.
    #[test]
    fn extended_width_decodes_a_magnitude_the_scaling_cannot_represent() {
        // The extended header carries a full byte width, so the decoder's own
        // output reaches the top of the range. Drive a real payload through
        // the decoder rather than asserting a property of `checked_mul`: what
        // matters is that this magnitude arrives from the wire.
        let mut b = PayloadBuilder::new();
        b.server_tag(0, 7);
        b.tick_extended(0, O_LAST_PRICE, 8, u64::MAX >> 1, false);
        let ticks = decode_ticks_35p(&b.build());
        assert_eq!(ticks.len(), 1, "the extended entry must decode");

        let mts: i64 = 1_000_000; // a one-cent tick against PRICE_SCALE 1e8
        assert!(
            ticks[0].magnitude.checked_mul(mts).is_none(),
            "a decoded magnitude of {} times min_tick_scaled leaves i64, which \
             is what the consumer has to detect",
            ticks[0].magnitude,
        );

        // The ordinary case still scales exactly, through the same path.
        let mut b2 = PayloadBuilder::new();
        b2.server_tag(0, 7);
        b2.tick(O_LAST_PRICE, 0, 2, 15_000, false);
        let ticks2 = decode_ticks_35p(&b2.build());
        assert_eq!(ticks2.len(), 1);
        assert_eq!(ticks2[0].magnitude.checked_mul(mts), Some(15_000_000_000));
    }
}

/// A tick stating no width takes no bit from the tick behind it.
///
/// The width is the whole value, sign bit included, so an entry stating none
/// has no sign bit either. Read as though it had one, it consumed a bit
/// belonging to whatever followed and published a zero of its own: the tick
/// behind it decoded a bit out of step, and its value and every value after
/// it in the message were wrong or gone.
#[test]
fn a_value_of_no_width_does_not_take_the_next_ticks_sign_bit() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 7);
    // An extended entry whose stated width is zero, with more to follow.
    b.push(31, 5);
    b.push(1, 1); // has_more
    b.push(0, 2); // raw_width, which an extended entry does not use
    b.push(4, 8); // extended tick type
    b.push(0, 8); // and no bytes of value
    // The tick behind it: one byte, positive, 42.
    b.tick(1, 0, 1, 42, false);

    let ticks = decode_ticks_35p(&b.build());
    assert!(
        ticks.iter().any(|t| t.magnitude == 42),
        "the entry stating no width took the following tick's sign bit: {ticks:?}",
    );
}

/// An entry states how far the decimal point moves in four bits and its width
/// in four more, not the width in a byte.
///
/// Read as one byte, the two multiply: two places out and four bytes wide read
/// as thirty-six bytes, which is wider than this decoder takes — so the entry
/// was stepped over by two hundred and eighty-eight bits instead of thirty-two
/// and every tick behind it was decoded out of step.
#[test]
fn a_shifted_entry_states_its_width_in_four_bits() {
    let mut b = PayloadBuilder::new();
    b.server_tag(0, 42);
    b.tick_shifted(1, O_CLOSE_PRICE, 2, 4, 1_250, false);
    b.tick(0, O_BID_PRICE, 4, 99, false);
    let ticks = decode_ticks_35p(&b.build());
    assert_eq!(ticks.len(), 2, "the entry behind it is still in step: {ticks:?}");
    assert_eq!(ticks[0].tick_type, O_CLOSE_PRICE);
    assert_eq!(ticks[0].magnitude, 1_250);
    assert_eq!(ticks[0].decimal_shift, 2, "and says how far out it is");
    assert_eq!(ticks[1].tick_type, O_BID_PRICE);
    assert_eq!(ticks[1].magnitude, 99);
    assert_eq!(ticks[1].decimal_shift, 0, "an ordinary entry states none");
}
