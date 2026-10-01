//! Scanner permissions and the names used in subscription notices.

use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct Permissions {
    instruments: Vec<Instrument>,
    layouts: Vec<(String, Vec<Filter>)>,
    filters: Vec<Filter>,
    scans: Vec<(String, String, Access)>,
    locations: Option<Vec<Location>>,
}

#[derive(Clone, Debug)]
struct Instrument {
    kind: String,
    filters: Vec<String>,
}

#[derive(Clone, Debug)]
struct Filter {
    id: String,
    codes: Vec<String>,
    components: Vec<String>,
    valid: bool,
    theme: bool,
    real_time_only: bool,
    label: String,
    access: Access,
}

#[derive(Clone, Debug)]
struct Location {
    code: String,
    name: String,
    access: Access,
    children: Vec<Location>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Level {
    #[default]
    Default,
    Disabled,
    Restricted,
    Allowed,
    Unrestricted,
}

#[derive(Clone, Debug, Default)]
struct Access {
    level: Level,
    features: Vec<String>,
    subscriptions: bool,
}

impl Access {
    fn prohibited(&self, enabled: &[String]) -> bool {
        !self.features.is_empty()
            && self.features.iter().all(|key| feature_state(key, enabled) == 0)
    }

    fn parse(value: &str) -> Self {
        let mut parts = value.split(';').filter(|part| !part.is_empty());
        let level = match parts.next().unwrap_or("") {
            "disabled" => Level::Disabled,
            "restricted" => Level::Restricted,
            "allowed" => Level::Allowed,
            "unrestricted" => Level::Unrestricted,
            _ => return Self::default(),
        };
        let mut access = Self { level, ..Self::default() };
        let mut feature = String::new();
        let flush = |value: &mut String, access: &mut Access| {
            if !value.is_empty() {
                let key = value.split(':').next().unwrap_or("").trim();
                if !key.is_empty() {
                    access.features.push(key.to_string());
                }
                value.clear();
            }
        };
        for part in parts.map(str::trim) {
            if let Some(key) = part.strip_prefix("f=") {
                flush(&mut feature, &mut access);
                feature.push_str(key);
            } else if let Some(ids) = part.strip_prefix("s=") {
                flush(&mut feature, &mut access);
                access.subscriptions |= !ids.trim().is_empty();
            } else if !feature.is_empty() {
                feature.push(';');
                feature.push_str(part);
            }
        }
        flush(&mut feature, &mut access);
        access
    }
}

impl Permissions {
    pub(crate) fn parse(xml: &str) -> Self {
        let roots = parse_elements(xml);
        let Some(root) = roots.iter().find(|node| node.name == "ScanParameterResponse") else {
            return Self::default();
        };
        let mut out = Self::default();
        if let Some(list) = root.child("InstrumentList") {
            for instrument in &list.children {
                out.instruments.push(Instrument {
                    kind: instrument.text("type"),
                    filters: comma_list(&instrument.text("filters")),
                });
            }
        }
        if let Some(list) = root.child("FilterList") {
            out.filters = filters(list);
        }
        if let Some(list) = root.child("ScannerLayoutList") {
            for layout in &list.children {
                if let Some(list) = layout
                    .children
                    .iter()
                    .find(|node| node.attribute("varName") == Some("specialFilters"))
                {
                    out.layouts.push((layout.text("instrument"), filters(list)));
                }
            }
        }
        if let Some(list) = root.child("ScanTypeList") {
            for scan in &list.children {
                let name = scan.text("displayName");
                out.scans.push((
                    scan.text("scanCode"),
                    if name.is_empty() { scan.text("searchName") } else { name },
                    Access::parse(&scan.text("access")),
                ));
            }
        }
        if let Some(tree) = root.child("LocationTree") {
            out.locations = Some(locations(tree));
        }
        out
    }

    fn has_code(
        &self,
        filter: &Filter,
        code: &str,
        visiting: &mut HashSet<String>,
        enabled: &[String],
    ) -> bool {
        if filter.codes.iter().any(|candidate| candidate == code) {
            return true;
        }
        if !visiting.insert(filter.id.clone()) {
            return false;
        }
        filter.components.iter().any(|id| {
            self.filters.iter().find(|filter| &filter.id == id && filter.valid).is_some_and(
                |filter| {
                    !filter.theme
                        && !filter.real_time_only
                        && (filter.id == "BT_RANKING_CRITERIA"
                            || !filter.access.prohibited(enabled))
                        && self.has_code(filter, code, visiting, enabled)
                },
            )
        })
    }

    pub(crate) fn notice(
        &self,
        sub: &super::ScannerSubscription,
        enabled: &[String],
    ) -> Option<(i32, String)> {
        let selected = sub.location_code.split([',', ';']).collect::<HashSet<_>>();
        let mut names = Vec::new();
        if let Some(locations) = &self.locations {
            selected_locations(locations, &selected, false, &mut names);
        }
        let restricted = !sub.location_code.is_empty() && !names.is_empty();
        let available = self.locations.is_some() && !restricted;
        let location_names = location_text(&names);
        let mut denied = Notice::default();
        let mut warned = Notice::default();
        if let Some((_, label, access)) =
            self.scans.iter().find(|(code, ..)| code == &sub.scan_code)
        {
            add_item(
                access,
                format!("Parameter:{label}"),
                false,
                available,
                restricted,
                &location_names,
                enabled,
                &mut denied,
                &mut warned,
            );
        }
        let ids = self
            .instruments
            .iter()
            .find(|instrument| instrument.kind == sub.instrument && instrument.kind != "Global")
            .map(|instrument| instrument.filters.as_slice())
            .unwrap_or(&[]);
        let special = self
            .layouts
            .iter()
            .find(|(kind, _)| kind == &sub.instrument)
            .map(|(_, filters)| filters.as_slice())
            .unwrap_or(&[]);
        let candidates = ids
            .iter()
            .filter_map(|id| {
                special.iter().chain(&self.filters).find(|filter| &filter.id == id && filter.valid)
            })
            .filter(|filter| {
                !filter.theme
                    && (filter.id == "BT_RANKING_CRITERIA" || !filter.access.prohibited(enabled))
            })
            .collect::<Vec<_>>();
        let mut seen = HashSet::new();
        for (code, _) in &sub.filters {
            if let Some(filter) = candidates
                .iter()
                .find(|filter| self.has_code(filter, code, &mut HashSet::new(), enabled))
                && seen.insert(&filter.id)
            {
                add_item(
                    &filter.access,
                    filter.label.clone(),
                    true,
                    available,
                    restricted,
                    &location_names,
                    enabled,
                    &mut denied,
                    &mut warned,
                );
            }
        }
        denied.finish(490, "You must subscribe for additional permissions to run scanner")
            .or_else(|| warned.finish(492, "You must subscribe for additional permissions to obtain precise results for scanner"))
    }
}

#[derive(Default)]
struct Notice {
    items: Vec<String>,
    filters: Vec<String>,
    reasons: BTreeSet<String>,
    locations: String,
}

impl Notice {
    fn finish(mut self, code: i32, base: &str) -> Option<(i32, String)> {
        if self.reasons.is_empty() {
            return None;
        }
        if !self.filters.is_empty() {
            self.items.push(format!("Filter:{}", self.filters.join(",")));
        }
        let reasons = self
            .reasons
            .into_iter()
            .map(|reason| {
                if reason == "Real-Time Market Data" && !self.locations.is_empty() {
                    format!("{reason}:{}", self.locations)
                } else {
                    reason
                }
            })
            .collect::<Vec<_>>();
        Some((code, format!("{base}:{};{}", self.items.join(","), reasons.join(","))))
    }
}

#[allow(clippy::too_many_arguments)]
fn add_item(
    access: &Access,
    label: String,
    filter: bool,
    available: bool,
    restricted: bool,
    locations: &str,
    enabled: &[String],
    denied: &mut Notice,
    warned: &mut Notice,
) {
    let prohibited = !filter && access.prohibited(enabled);
    let target = match access.level {
        _ if prohibited => denied,
        Level::Disabled => denied,
        Level::Default if !available => denied,
        Level::Restricted => warned,
        Level::Allowed if restricted => warned,
        _ => return,
    };
    if filter {
        target.filters.push(label);
    } else {
        target.items.push(label);
    }
    for key in &access.features {
        if feature_state(key, enabled) == 1
            && let Some(label) = feature_label(key)
        {
            target.reasons.insert(label.to_string());
        }
    }
    if access.subscriptions {
        target.reasons.insert("Real-Time Market Data".to_string());
    }
    if restricted && matches!(access.level, Level::Default | Level::Allowed) {
        target.reasons.insert("Real-Time Market Data".to_string());
        if target.locations.is_empty() {
            target.locations = locations.to_string();
        }
    }
}

fn feature_state(key: &str, enabled: &[String]) -> u8 {
    if feature_label(key).is_none() {
        return 0;
    }
    enabled
        .iter()
        .find_map(|token| {
            if token == key {
                Some(2)
            } else if token.strip_prefix(key).is_some_and(|suffix| suffix.starts_with(':')) {
                Some(1)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn feature_label(key: &str) -> Option<&'static str> {
    Some(match key {
        "WSH" => "Wall Street Horizons",
        "MOODY" => "Moody's US Bond Ratings (Corporates and Municipals)",
        "FITCH" => "Fitch US Bond Ratings (Corporates and Municipals)",
        "SP" => "S&P US Bond Ratings (Corporates and Municipals)",
        "REUTFUND" => "Refinitiv Worldwide Fundamentals",
        "CUSIP" => "CUSIP",
        "PERMST" => "Permanent token",
        "DATABOOST" => "DATA BOOST INFO",
        "MSOWN" => "Morningstar data",
        "PERMST1" => "Permanent token SHA1",
        "ORDER" => "Order",
        "MD" => "Market data",
        "DAYLINEUP" => "Daily Lineup",
        "BBTV" => "Bloomberg TV",
        "BBTV_ASIA" => "Bloomberg TV Asia",
        "VNDMDL" => "%MODEL_MARKETPLACE%",
        "IBIS" => "IB Information System",
        _ => return None,
    })
}

fn locations(tree: &Element<'_>) -> Vec<Location> {
    tree.children
        .iter()
        .filter(|node| node.name == "Location")
        .map(|node| Location {
            code: node.text("locationCode"),
            name: node.text("displayName"),
            access: Access::parse(&node.text("access")),
            children: node.child("LocationTree").map(locations).unwrap_or_default(),
        })
        .collect()
}

fn selected_locations(
    locations: &[Location],
    selected: &HashSet<&str>,
    parent: bool,
    names: &mut Vec<String>,
) {
    for location in locations {
        let selected_here = parent || selected.contains(location.code.as_str());
        if location.children.is_empty() {
            if selected_here && location.access.level != Level::Default {
                names.push(location.name.clone());
            }
        } else {
            selected_locations(&location.children, selected, selected_here, names);
        }
    }
}

#[derive(Default)]
struct Element<'a> {
    name: &'a str,
    attributes: &'a str,
    value: String,
    children: Vec<Element<'a>>,
}

impl<'a> Element<'a> {
    fn child(&self, name: &str) -> Option<&Self> {
        self.children.iter().find(|node| node.name == name)
    }
    fn text(&self, name: &str) -> String {
        self.child(name).map(|node| node.value.clone()).unwrap_or_default()
    }
    fn attribute(&self, name: &str) -> Option<&'a str> {
        crate::control::xml::attr(self.attributes, name)
    }
}

fn unescape(mut value: &str) -> String {
    let mut out = String::new();
    while let Some(at) = value.find('&') {
        out.push_str(&value[..at]);
        value = &value[at..];
        let Some(end) = value.find(';') else { break };
        let entity = &value[1..end];
        let ch = match entity {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "amp" => Some('&'),
            _ => entity
                .strip_prefix("#x")
                .and_then(|n| u32::from_str_radix(n, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|n| n.parse().ok()))
                .and_then(char::from_u32),
        };
        if let Some(ch) = ch {
            out.push(ch);
        } else {
            out.push_str(&value[..=end]);
        }
        value = &value[end + 1..];
    }
    out.push_str(value);
    out
}

fn comma_list(value: &str) -> Vec<String> {
    value.split(',').filter(|value| !value.is_empty()).map(str::to_string).collect()
}

fn filters(list: &Element<'_>) -> Vec<Filter> {
    list.children
        .iter()
        .map(|filter| {
            let fields = filter
                .children
                .iter()
                .filter(|node| {
                    matches!(node.name, "AbstractField" | "TripleComboField" | "ThemeField")
                })
                .collect::<Vec<_>>();
            let kind =
                filter.attribute("type").unwrap_or(filter.name).rsplit('.').next().unwrap_or("");
            let id = filter.text("id");
            let field = |name| {
                fields
                    .iter()
                    .find(|field| {
                        field.attribute("varName") == Some(name) || field.text("varName") == name
                    })
                    .copied()
            };
            let complete = |field: &Element<'_>| {
                !field.text("code").is_empty() && !field.text("displayName").is_empty()
            };
            let valid = match kind {
                "RangeFilter" => {
                    !id.is_empty()
                        && field("min").is_some_and(complete)
                        && field("max").is_some_and(complete)
                }
                "SimpleFilter" => !id.is_empty() && field("field").is_some_and(complete),
                "ThemeFilter" => field("field").is_some(),
                _ => true,
            };
            let mut codes = Vec::new();
            if matches!(kind, "TripleComboFilter" | "ThemeFilter") {
                codes.push(id.clone());
            }
            for field in &fields {
                codes.push(field.text("code"));
                if kind != "RangeFilter" {
                    codes.push(field.text("codeNot"));
                }
            }
            codes.retain(|code| !code.is_empty());
            let mut label = if kind == "RangeFilter" {
                let label = field("max").map(|field| field.text("displayName")).unwrap_or_default();
                match label.to_ascii_lowercase().find(" below") {
                    Some(at) => label[..at].to_string(),
                    None => label.to_string(),
                }
            } else if kind == "VirtualFilter" {
                filter.text("displayName")
            } else {
                fields.first().map(|field| field.text("displayName")).unwrap_or_default()
            };
            if kind == "RangeFilter" && filter.text("reuters") == "yes" {
                label.push_str(" (Refinitiv)");
            }
            let components = filter
                .child("FilterComponents")
                .map(|list| list.children.iter().map(|node| node.text("code")).collect())
                .unwrap_or_default();
            Filter {
                theme: if kind == "TripleComboFilter" {
                    field("triField").is_some_and(|field| field.text("code") == "stockThemeIs")
                } else {
                    id == "STOCKTHEME" || kind == "ThemeFilter"
                },
                real_time_only: {
                    let value = filter.text("realTimeOnly");
                    value == "1"
                        || value.eq_ignore_ascii_case("true")
                        || value.eq_ignore_ascii_case("yes")
                },
                id,
                codes,
                components,
                valid,
                label,
                access: Access::parse(&filter.text("access")),
            }
        })
        .collect()
}

fn location_text(names: &[String]) -> String {
    let mut text = names.join(", ").chars().collect::<Vec<_>>();
    let mut at = 80;
    while at < text.len() {
        if let Some(comma) = text[at..].iter().position(|ch| *ch == ',') {
            text.insert(at + comma + 1, '\n');
        }
        at += 80;
    }
    text.into_iter().collect()
}

fn parse_elements(mut xml: &str) -> Vec<Element<'_>> {
    let mut stack: Vec<Element<'_>> = Vec::new();
    let mut roots = Vec::new();
    while let Some(at) = xml.find('<') {
        if let Some(node) = stack.last_mut() {
            node.value.push_str(&unescape(&xml[..at]));
        }
        xml = &xml[at..];
        if let Some(comment) = xml.strip_prefix("<!--") {
            let Some(end) = comment.find("-->") else { break };
            xml = &comment[end + 3..];
            continue;
        }
        if let Some(value) = xml.strip_prefix("<![CDATA[") {
            let Some(end) = value.find("]]>") else { break };
            if let Some(node) = stack.last_mut() {
                node.value.push_str(&value[..end]);
            }
            xml = &value[end + 3..];
            continue;
        }
        let Some(end) = xml.find('>') else { break };
        let header = &xml[1..end];
        xml = &xml[end + 1..];
        if header.starts_with(['?', '!']) {
            continue;
        }
        let node = if let Some(name) = header.strip_prefix('/') {
            let Some(node) = stack.pop().filter(|node| node.name == name.trim()) else {
                return Vec::new();
            };
            node
        } else {
            let trimmed = header.trim_end_matches('/').trim_end();
            let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
            let node = Element {
                name: &trimmed[..end],
                attributes: &trimmed[end..],
                ..Default::default()
            };
            if !header.ends_with('/') {
                stack.push(node);
                continue;
            }
            node
        };
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            roots.push(node);
        }
    }
    if stack.is_empty() { roots } else { Vec::new() }
}
