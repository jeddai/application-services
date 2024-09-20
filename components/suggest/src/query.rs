/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{LabeledTimingSample, Suggestion, SuggestionProvider, SuggestionProviderConstraints};

/// A query for suggestions to show in the address bar.
#[derive(Clone, Debug, Default, uniffi::Record)]
pub struct SuggestionQuery {
    pub keyword: String,
    pub providers: Vec<SuggestionProvider>,
    #[uniffi(default = None)]
    pub provider_constraints: Option<SuggestionProviderConstraints>,
    #[uniffi(default = None)]
    pub limit: Option<i32>,
}

#[derive(uniffi::Record)]
pub struct QueryWithMetricsResult {
    pub suggestions: Vec<Suggestion>,
    /// Samples for the `suggest.query_time` metric
    pub query_times: Vec<LabeledTimingSample>,
}

impl SuggestionQuery {
    // Builder style methods for creating queries (mostly used by the test code)

    pub fn all_providers(keyword: &str) -> Self {
        Self {
            keyword: keyword.to_string(),
            providers: Vec::from(SuggestionProvider::all()),
            ..Self::default()
        }
    }

    pub fn with_providers(keyword: &str, providers: Vec<SuggestionProvider>) -> Self {
        Self {
            keyword: keyword.to_string(),
            providers,
            ..Self::default()
        }
    }

    pub fn all_providers_except(keyword: &str, provider: SuggestionProvider) -> Self {
        Self::with_providers(
            keyword,
            SuggestionProvider::all()
                .into_iter()
                .filter(|p| *p != provider)
                .collect(),
        )
    }

    pub fn amp(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Amp],
            ..Self::default()
        }
    }

    pub fn wikipedia(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Wikipedia],
            ..Self::default()
        }
    }

    pub fn amp_mobile(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::AmpMobile],
            ..Self::default()
        }
    }

    pub fn amo(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Amo],
            ..Self::default()
        }
    }

    pub fn pocket(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Pocket],
            ..Self::default()
        }
    }

    pub fn yelp(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Yelp],
            ..Self::default()
        }
    }

    pub fn mdn(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Mdn],
            ..Self::default()
        }
    }

    pub fn fakespot(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Fakespot],
            ..Self::default()
        }
    }

    pub fn weather(keyword: &str) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Weather],
            ..Self::default()
        }
    }

    pub fn exposure(keyword: &str, suggestion_types: &[&str]) -> Self {
        Self {
            keyword: keyword.into(),
            providers: vec![SuggestionProvider::Exposure],
            provider_constraints: Some(SuggestionProviderConstraints {
                exposure_suggestion_types: Some(
                    suggestion_types.iter().map(|s| s.to_string()).collect(),
                ),
            }),
            ..Self::default()
        }
    }

    pub fn limit(self, limit: i32) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }

    // Other Functionality

    /// Parse the `keyword` field into a set of keywords.
    ///
    /// This is used when passing the keywords into an FTS search.  It:
    ///   - Strips out any `():^*"` chars.  These are typically used for advanced searches, which
    ///     we don't support and it would be weird to only support for FTS searches, which
    ///     currently means Fakespot searches.
    ///   - Splits on whitespace to get a list of individual keywords
    ///
    pub(crate) fn parse_keywords(&self) -> Vec<&str> {
        self.keyword
            .split([' ', '(', ')', ':', '^', '*', '"'])
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Create an FTS query term for our keyword(s)
    pub(crate) fn fts_query(&self) -> String {
        let keywords = self.parse_keywords();
        if keywords.is_empty() {
            return String::from(r#""""#);
        }
        // Quote each term from `query` and join them together
        let mut fts_query = keywords
            .iter()
            .map(|keyword| format!(r#""{keyword}""#))
            .collect::<Vec<_>>()
            .join(" ");
        // If the input is > 3 characters, and there's no whitespace at the end.
        // We want to append a `*` char to the end to do a prefix match on it.
        let total_chars = keywords.iter().fold(0, |count, s| count + s.len());
        let query_ends_in_whitespace = self.keyword.ends_with(' ');
        if (total_chars > 3) && !query_ends_in_whitespace {
            fts_query.push('*');
        }
        fts_query
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_parse_keywords(input: &str, expected: Vec<&str>) {
        let query = SuggestionQuery::all_providers(input);
        assert_eq!(query.parse_keywords(), expected);
    }

    #[test]
    fn test_quote() {
        check_parse_keywords("foo", vec!["foo"]);
        check_parse_keywords("foo bar", vec!["foo", "bar"]);
        // Special chars should be stripped
        check_parse_keywords("\"foo()* ^bar:\"", vec!["foo", "bar"]);
        // test some corner cases
        check_parse_keywords("", vec![]);
        check_parse_keywords(" ", vec![]);
        check_parse_keywords("   foo     bar       ", vec!["foo", "bar"]);
        check_parse_keywords("foo:bar", vec!["foo", "bar"]);
    }

    fn check_fts_query(input: &str, expected: &str) {
        let query = SuggestionQuery::all_providers(input);
        assert_eq!(query.fts_query(), expected);
    }

    #[test]
    fn test_fts_query() {
        // String with < 3 chars shouldn't get a prefix query
        check_fts_query("r", r#""r""#);
        check_fts_query("ru", r#""ru""#);
        check_fts_query("run", r#""run""#);
        // After 3 chars, we should append `*` to the last term to make it a prefix query
        check_fts_query("runn", r#""runn"*"#);
        check_fts_query("running", r#""running"*"#);
        // The total number of chars is counted, not the number of chars in the last term
        check_fts_query("running s", r#""running" "s"*"#);
        // if the input ends in whitespace, then don't do a prefix query
        check_fts_query("running ", r#""running""#);
        // Special chars are filtered out
        check_fts_query("running*\"()^: s", r#""running" "s"*"#);
        check_fts_query("running *\"()^: s", r#""running" "s"*"#);
        // Special chars shouldn't count towards the input size when deciding whether to do a
        // prefix query or not
        check_fts_query("r():", r#""r""#);
        // Test empty strings
        check_fts_query("", r#""""#);
        check_fts_query(" ", r#""""#);
        check_fts_query("()", r#""""#);
    }
}
