//! Network blocking policy facade for Aurexalis.
//!
//! The current matcher is deliberately small and deterministic. It preserves
//! the public policy shape that will later wrap `adblock-rust`.

#![forbid(unsafe_code)]

use aurexalis_core::{NetworkRequest, ResourceKind};
use std::fmt;

#[derive(Debug)]
pub enum BlockerError {
    EmptyFilterList,
}

impl fmt::Display for BlockerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockerError::EmptyFilterList => formatter.write_str("filter list is empty"),
        }
    }
}

impl std::error::Error for BlockerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDecision {
    Allow,
    AllowByException { rule: String },
    Block { rule: String },
}

#[derive(Debug, Default)]
pub struct BlockerEngine {
    rules: Vec<String>,
}

impl BlockerEngine {
    pub fn from_filter_lists(lists: &[String]) -> Result<Self, BlockerError> {
        let rules = lists
            .iter()
            .flat_map(|list| list.lines())
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('!'))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if rules.is_empty() {
            return Err(BlockerError::EmptyFilterList);
        }

        Ok(Self { rules })
    }

    pub fn check(&self, request: &NetworkRequest) -> BlockDecision {
        for rule in &self.rules {
            if rule.starts_with("@@") && matches_rule(&rule[2..], request) {
                return BlockDecision::AllowByException { rule: rule.clone() };
            }
        }

        for rule in &self.rules {
            if matches_rule(rule, request) {
                return BlockDecision::Block { rule: rule.clone() };
            }
        }

        BlockDecision::Allow
    }
}

fn matches_rule(rule: &str, request: &NetworkRequest) -> bool {
    let (pattern, options) = split_options(rule);

    if !options_match(options, request) {
        return false;
    }

    if pattern.starts_with("||") {
        let needle = pattern
            .trim_start_matches("||")
            .trim_end_matches('^')
            .trim_end_matches('/');

        return request
            .host()
            .is_some_and(|domain| domain == needle || domain.ends_with(&format!(".{needle}")));
    }

    request.url.contains(pattern)
}

fn split_options(rule: &str) -> (&str, Option<&str>) {
    match rule.split_once('$') {
        Some((pattern, options)) => (pattern, Some(options)),
        None => (rule, None),
    }
}

fn options_match(options: Option<&str>, request: &NetworkRequest) -> bool {
    let Some(options) = options else {
        return true;
    };

    for option in options.split(',').map(str::trim) {
        match option {
            "script" if request.kind != ResourceKind::Script => return false,
            "image" if request.kind != ResourceKind::Image => return false,
            "stylesheet" if request.kind != ResourceKind::Stylesheet => return false,
            "third-party" if !request.is_third_party() => return false,
            _ => {}
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurexalis_core::ResourceKind;

    fn request(url: &str, source: Option<&str>, kind: ResourceKind) -> NetworkRequest {
        NetworkRequest::parse(url, source, kind).expect("test request should parse")
    }

    #[test]
    fn blocks_domain_rule() {
        let engine = BlockerEngine::from_filter_lists(&["||ads.example.com^".to_owned()])
            .expect("rules should load");
        let decision = engine.check(&request(
            "https://ads.example.com/banner.js",
            Some("https://site.test"),
            ResourceKind::Script,
        ));

        assert_eq!(
            decision,
            BlockDecision::Block {
                rule: "||ads.example.com^".to_owned()
            }
        );
    }

    #[test]
    fn honors_exception_before_block() {
        let engine = BlockerEngine::from_filter_lists(&[
            "||ads.example.com^".to_owned(),
            "@@||ads.example.com^".to_owned(),
        ])
        .expect("rules should load");

        assert_eq!(
            engine.check(&request(
                "https://ads.example.com/allowed.js",
                Some("https://site.test"),
                ResourceKind::Script,
            )),
            BlockDecision::AllowByException {
                rule: "@@||ads.example.com^".to_owned()
            }
        );
    }

    #[test]
    fn applies_resource_type_option() {
        let engine =
            BlockerEngine::from_filter_lists(&["tracker.js$script,third-party".to_owned()])
                .expect("rules should load");

        assert!(matches!(
            engine.check(&request(
                "https://cdn.test/tracker.js",
                Some("https://site.test"),
                ResourceKind::Script,
            )),
            BlockDecision::Block { .. }
        ));

        assert_eq!(
            engine.check(&request(
                "https://site.test/tracker.js",
                Some("https://site.test"),
                ResourceKind::Script,
            )),
            BlockDecision::Allow
        );
    }

    #[test]
    fn rejects_empty_lists() {
        let error = BlockerEngine::from_filter_lists(&["! only a comment".to_owned()])
            .expect_err("empty rules should fail");

        assert!(matches!(error, BlockerError::EmptyFilterList));
    }
}
