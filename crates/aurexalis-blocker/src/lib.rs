use aurexalis_core::NetworkRequest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockerError {
    #[error("filter list is empty")]
    EmptyFilterList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDecision {
    Allow,
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
        // This placeholder keeps the policy API stable before wiring adblock-rust.
        for rule in &self.rules {
            if rule.starts_with("||") {
                let needle = rule.trim_start_matches("||").trim_end_matches('^');
                if request.url.domain().is_some_and(|domain| domain.contains(needle)) {
                    return BlockDecision::Block { rule: rule.clone() };
                }
            }
        }

        BlockDecision::Allow
    }
}

