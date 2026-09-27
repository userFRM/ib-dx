//! Reading and writing the venue's XML.
//!
//! Several of the venue's answers arrive as small XML documents — a
//! fundamentals report, a histogram, a scanner result, a news article, a bar
//! query's own envelope. They are read by name rather than parsed, because
//! what is wanted from each is a handful of known tags and a parser would be a
//! dependency and a shape to keep in step with the venue's. The one document
//! this client writes is here for the same reason.

/// The text between `<tag>` and `</tag>`, or nothing where the pair is absent.
///
/// Borrows from the document rather than copying: these are read once, on the
/// hot loop, and a reply carrying a hundred rows would otherwise allocate a
/// string per field to throw it away again.
///
/// The first pair wins. A document nesting the same name inside itself would
/// need a parser, and none of the replies read here does.
pub fn tag<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(&xml[start..end])
}

/// The value of an element's `name="value"` attribute, or nothing where the
/// attribute is absent.
///
/// The first match wins, for the same reason [`tag`] takes it: none of the
/// replies read here nests an attribute's name inside a value.
pub fn attr<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let start = xml.find(&format!("{name}=\""))? + name.len() + 2;
    let end = xml[start..].find('"')? + start;
    Some(&xml[start..end])
}

/// Every element of a name in the document, whole — its open, which may
/// carry attributes, its content and its close — in the order they appear.
///
/// For the answers that repeat an element — one per dividend, one per rate —
/// where [`tag`] would read the first and stop, and an attribute of one is
/// part of what it states. Nesting the same name inside itself would need a
/// parser, and none of the replies read here does.
pub fn elements<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut out = Vec::new();
    let mut at = 0usize;
    while let Some(found) = xml[at..].find(&open) {
        let tag = at + found;
        let after = tag + open.len();
        // The element's own name, and not one that merely begins with it: the
        // name is followed by its end or by an attribute's whitespace. One
        // that closes itself states no content and is passed over.
        if !xml[after..].starts_with(['>', ' ', '\t', '\r', '\n']) {
            at = after;
            continue;
        }
        let Some(tag_end) = xml[after..].find('>') else { break };
        if xml[after..after + tag_end].ends_with('/') {
            at = after + tag_end + 1;
            continue;
        }
        let start = after + tag_end + 1;
        let Some(end) = xml[start..].find(&close) else { break };
        out.push(&xml[tag..start + end + close.len()]);
        at = start + end + close.len();
    }
    out
}

/// Withdraw a query the venue is still answering, by the id it was asked under.
///
/// The venue takes the same document for every kind of query — fundamentals,
/// corporate actions — because what it cancels is the query, not the subject.
pub fn cancel_query(query_id: &str) -> String {
    format!(
        "<ListOfCancelQueries>\
         <CancelQuery>\
         <id>{query_id}</id>\
         </CancelQuery>\
         </ListOfCancelQueries>",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What every caller asks of it, and the two ways a document can not
    /// answer: a name that is not there, and one that opens without closing.
    #[test]
    fn a_named_tag_reads_back_and_a_missing_one_reads_as_nothing() {
        let doc = "<HistoricalDataRequest><id>hist_7</id><bar>5 mins</bar></HistoricalDataRequest>";
        assert_eq!(tag(doc, "id"), Some("hist_7"));
        assert_eq!(tag(doc, "bar"), Some("5 mins"));
        assert_eq!(tag(doc, "absent"), None);
        assert_eq!(tag("<id>never closed", "id"), None);
        assert_eq!(tag("<id></id>", "id"), Some(""), "stated and empty is not absent");
    }

    /// The first pair wins, which is what every reply read here needs and all
    /// that this can promise without being a parser.
    #[test]
    fn the_first_pair_is_the_one_read() {
        assert_eq!(tag("<a>one</a><a>two</a>", "a"), Some("one"));
    }

    /// And where the answer repeats an element, every one of them in order.
    #[test]
    fn a_repeated_element_reads_back_in_the_order_it_was_stated() {
        assert_eq!(elements("<a>one</a><b>x</b><a>two</a>", "a"), ["<a>one</a>", "<a>two</a>"]);
        assert_eq!(
            elements("<a></a><a>two</a>", "a"), ["<a></a>", "<a>two</a>"],
            "stated and empty counts",
        );
        assert!(elements("<a>never closed<a>nor this", "a").is_empty(), "neither pair closes");
        assert!(elements("<b>x</b>", "a").is_empty());
        // An element's attributes are part of what it states.
        assert_eq!(elements("<a s=\"true\">one</a>", "a"), ["<a s=\"true\">one</a>"]);
        // A name that merely begins with the name is not an element of it, and
        // one that closes itself states no content.
        assert!(elements("<ab>x</ab>", "a").is_empty());
        assert_eq!(elements("<a/><a>two</a>", "a"), ["<a>two</a>"]);
    }
}
