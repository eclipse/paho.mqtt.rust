// topic_matcher.rs
//
// Code to match MQTT topics to filters that may contain wildcards.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2024 Altair Bueno <altair@uma.es>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v20.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Altair Bueno - initial implementation and documentation
 *******************************************************************************/

//! Code to match MQTT topics to filters that may contain wildcards.

/// Checks if a filter matches a given topic.
///
/// # Example
///
/// ```
/// use paho_mqtt::topic_matcher::matches;
///
/// assert!(matches("a/+/c", "a/b/c"));
/// assert!(matches("a/#", "a/b/d"));
/// ```
pub fn matches(filter: &str, topic: &str) -> bool {
    matches_iter(filter.split('/'), topic.split('/'))
}

/// Checks if a (splitted) filter matches a given (splitted) topic.
///
/// # Example
///
/// ```
/// use paho_mqtt::topic_matcher::matches_iter;
///
/// assert!(matches_iter(["a", "+", "c"], ["a", "b", "c"]));
/// assert!(matches_iter(["a", "#"], ["a", "b", "d"]));
/// ```
pub fn matches_iter<'a>(
    filter: impl IntoIterator<Item = &'a str>,
    topic: impl IntoIterator<Item = &'a str>,
) -> bool {
    let mut filter = filter.into_iter().peekable();
    let mut topic = topic.into_iter().peekable();

    // See https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901246
    if matches!(filter.peek(), Some(&"#" | &"+"))
        && matches!(topic.peek(), Some(x) if x.starts_with('$'))
    {
        return false;
    }

    loop {
        let filter_level = filter.next();
        let topic_level = topic.next();
        match (filter_level, topic_level) {
            // Exhausted both filter and topic
            (None, None) => return true,
            // Wildcard on filter
            (Some("#"), _) => return true,
            // Single level wildcard on filter
            (Some("+"), Some(_)) => continue,
            // Equal levels
            (Some(filter), Some(topic)) if filter == topic => continue,
            // Otherwise, no match
            _ => return false,
        }
    }
}

/// Extension trait for map types and tuple iterators that allows to filter
/// entries by matching a MQTT topic.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use std::collections::HashSet;
/// use paho_mqtt::topic_matcher::TopicMatcher as _;
///
/// let mut matcher = HashMap::<&str, &str>::new();
/// matcher.insert("00:00:00:00:00:00/+/+/rpc", "_/device_type/systemid/_");
/// matcher.insert("00:00:00:00:00:00/+/+/+/rpc", "_/device_type/systemid/zoneid/_");
/// matcher.insert("00:00:00:00:00:00/+/rpc", "_/device_type/_");
/// matcher.insert("00:00:00:00:00:00/rpc", "");
///
/// let topic = "00:00:00:00:00:00/humidifier/1/rpc";
/// let matches: HashSet<_> = matcher.matches(topic).collect();
/// assert_eq!(
///    matches,
///   HashSet::from([("00:00:00:00:00:00/+/+/rpc", "_/device_type/systemid/_")])
/// );
/// ```
pub trait TopicMatcherExt {
    /// The key type returned by the iterator.
    type Key;
    /// The value type returned by the iterator.
    type Value;

    /// Matches the given topic against the keys of the map and returns an
    /// iterator over the matching entries. Keys of the map are expected to
    /// be MQTT topic patterns and may contain wildcards.
    fn matches<'topic>(
        self,
        topic: &'topic str,
    ) -> impl Iterator<Item = (Self::Key, Self::Value)> + 'topic
    where
        Self: 'topic;
}

impl<K, V, C> TopicMatcherExt for C
where
    C: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
{
    type Key = K;
    type Value = V;

    fn matches<'topic>(
        self,
        topic: &'topic str,
    ) -> impl Iterator<Item = (Self::Key, Self::Value)> + 'topic
    where
        Self: 'topic,
    {
        self.into_iter()
            .filter(move |(pattern, _)| matches(pattern.as_ref(), topic))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assert_that_no_wildcards_matches() {
        assert!(matches("a/b/c", "a/b/c"));
        assert!(matches("foo/bar", "foo/bar"));
    }
    #[test]
    fn assert_that_plus_wildcard_matches() {
        assert!(matches("a/+/c", "a/b/c"));
        assert!(matches("foo/+/baz", "foo/bar/baz"));
    }
    #[test]
    fn assert_that_leading_plus_wildcard_matches() {
        assert!(matches("+/b/c", "a/b/c"));
    }
    #[test]
    fn assert_that_trailing_plus_wildcard_matches() {
        assert!(matches("a/b/+", "a/b/c"));
        assert!(matches("foo/+", "foo/bar"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_none_level() {
        assert!(matches("a/b/#", "a/b"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_single_level() {
        assert!(matches("a/b/#", "a/b/c"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_multiple_levels() {
        assert!(matches("a/b/#", "a/b/c/d"));
    }
    #[test]
    fn assert_that_single_hash_matches_all() {
        assert!(matches("#", "foo/bar/baz"));
        assert!(matches("#", "/foo/bar"));
        assert!(matches("/#", "/foo/bar"));
    }
    #[test]
    fn assert_that_plus_and_hash_wildcards_matches() {
        assert!(matches("foo/+/#", "foo/bar/baz"));
        assert!(matches("A/B/+/#", "A/B/B/C"));
    }
    #[test]
    fn assert_that_sys_topic_matches() {
        assert!(matches("$SYS/bar", "$SYS/bar"));
    }
    #[test]
    fn assert_that_non_first_levels_with_dollar_sign_matches_hash_wildcard() {
        assert!(matches("foo/#", "foo/$bar"));
    }
    #[test]
    fn assert_that_non_first_levels_with_dollar_sign_matches_plus_wildcard() {
        assert!(matches("foo/+/baz", "foo/$bar/baz"));
    }
    #[test]
    fn assert_that_different_levels_does_not_match() {
        assert!(!matches("test/6/#", "test/3"));
        assert!(!matches("foo/+/baz", "foo/bar/bar"));
        assert!(!matches("foo/+/#", "fo2/bar/baz"));
        assert!(!matches("$BOB/bar", "$SYS/bar"));
    }
    #[test]
    fn assert_that_longer_topics_does_not_match() {
        assert!(!matches("foo/bar", "foo"));
    }
    #[test]
    fn assert_that_plus_wildcard_does_not_match_multiple_levels() {
        assert!(!matches("foo/+", "foo/bar/baz"));
    }
    #[test]
    fn assert_that_leading_slash_with_hash_wildcard_does_not_match_normal_topic() {
        assert!(!matches("/#", "foo/bar"));
    }
    #[test]
    fn assert_that_hash_wildcard_does_not_match_an_internal_topic() {
        assert!(!matches("#", "$SYS/bar"));
    }
    #[test]
    fn assert_that_plus_wildcard_does_not_match_an_internal_topic() {
        assert!(!matches("+/bar", "$SYS/bar"));
    }
}
