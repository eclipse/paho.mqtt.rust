// topic_matcher.rs
//
// Code to match MQTT topics to filters that may contain wildcards.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2021-2024 Frank Pagliughi <fpagliughi@mindspring.com>
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
 *    Frank Pagliughi - initial TopicFilter trie collection
 *    Altair Bueno - TopicFilterExt trait and topic matches functions
 *******************************************************************************/

//! Code to match MQTT topics to filters that may contain wildcards.

use std::{
    collections::HashMap,
    str::Split,
};

/////////////////////////////////////////////////////////////////////////////
// Utility functions

/// Checks if a filter matches a given topic.
///
/// # Example
///
/// ```
/// use paho_mqtt::topic_matcher::topic_matches;
///
/// assert!(topic_matches("a/+/c", "a/b/c"));
/// assert!(topic_matches("a/#", "a/b/d"));
/// ```
pub fn topic_matches(filter: &str, topic: &str) -> bool {
    topic_matches_iter(filter.split('/'), topic.split('/'))
}

/// Checks if a (splitted) filter matches a given (splitted) topic.
///
/// # Example
///
/// ```
/// use paho_mqtt::topic_matcher::topic_matches_iter;
///
/// assert!(topic_matches_iter(["a", "+", "c"], ["a", "b", "c"]));
/// assert!(topic_matches_iter(["a", "#"], ["a", "b", "d"]));
/// ```
pub fn topic_matches_iter<'a>(
    filter: impl IntoIterator<Item = &'a str>,
    topic: impl IntoIterator<Item = &'a str>,
) -> bool {
    let mut filter = filter.into_iter().peekable();
    let mut topic = topic.into_iter().peekable();

    // Topics starting with '$' don't match a wildcard in the first field.
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
/// use std::collections::{HashMap, HashSet};
/// use paho_mqtt::topic_matcher::TopicMatcherExt as _;
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
    /// be MQTT topic filter patterns and may contain wildcards.
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
            .filter(move |(pattern, _)| topic_matches(pattern.as_ref(), topic))
    }
}

/////////////////////////////////////////////////////////////////////////////
// Node for TopicMatcher

/// A single node in the topic matcher collection.
///
/// A terminal (leaf) node has some `content`, whereas intermediate nodes
/// do not. We also cache the full topic at the leaf. This should allow for
/// more efficient searches through the collection, so that the iterators
/// don't have to keep the stack of keys that lead down to the final leaf.
///
/// Note that although we could put the wildcard keys into the `children`
/// map, we specifically have separate fields for them. That allows us to
/// have separate mutable references for each, allowing for a mutable
/// iterator.
#[derive(Debug)]
struct Node<T> {
    /// The value that matches the topic at this node, if any.
    /// This includes a ached value of the filter.
    value: Option<(String, T)>,
    /// The explicit, non-wildcard child nodes mapped by the next field of
    /// the topic.
    children: HashMap<Box<str>, Node<T>>,
}

impl<T> Node<T> {
    /// Determines if the node does not contain a value.
    ///
    /// This is a relatively simplistic implementation indicating that the
    /// node's content and children are empty. Technically, the node could
    /// contain a collection of children that are empty, which might be
    /// considered an "empty" state. But not here.
    fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }

    /// Gets an iterator for the node and all its children.
    //fn iter<'a>(&'a self) -> NodeIter<'a, T> {
    fn iter(&self) -> NodeIter<T> {
        Box::new(
            self.value
                .iter()
                .chain(self.children.values().map(|n| n.iter()).flatten()),
        )
    }

    /// Gets a mutable iterator for the node and all its children.
    fn iter_mut(&mut self) -> NodeIterMut<T> {
        Box::new(
            self.value
                .iter_mut()
                .chain(self.children.values_mut().map(|n| n.iter_mut()).flatten()),
        )
    }
}

// We manually implement Default, otherwise the derived one would
// require T: Default.

impl<T> Default for Node<T> {
    /// Creates a default, empty node.
    fn default() -> Self {
        Node {
            value: None,
            children: HashMap::new(),
        }
    }
}

/// An iterator to visit all values in a node and its children.
type NodeIter<'a, T> = Box<dyn Iterator<Item = &'a (String, T)> + 'a>;

/// A mutable iterator to visit all values in a node and its children.
type NodeIterMut<'a, T> = Box<dyn Iterator<Item = &'a mut (String, T)> + 'a>;

/////////////////////////////////////////////////////////////////////////////
// TopicMatcher

/// A collection of topic filters to arbitrary objects.
///
/// This can be used to get an iterator to all items that have a filter that
/// matches a topic. To test against a single filter, see
/// [`TopicFilter`](crate::TopicFilter). This collection is more commonly used
/// when there are a nuber of filters and each needs to be associated with a
/// particular action or piece of data. Note, though, that a single incoming
/// topic could match against several items in the collection. For example,
/// the topic:
///     data/temperature/engine
///
/// Could match against the filters:
///     data/temperature/engine
///     data/temperature/#
///     data/+/engine
///
/// Thus, the collection gives an iterator for the items matching a topic.
///
/// A common use for this would be to store callbacks to proces incoming
/// messages based on topics.
///
/// This code was adapted from the Eclipse Python `MQTTMatcher` class:
/// <https://github.com/eclipse/paho.mqtt.python/blob/master/src/paho/mqtt/matcher.py>
///
/// which use a prefix tree (trie) to store the values.
///

/// A collection of topic filters that can compare against a topic to
/// produce an iterator of all matched items.
///
/// This is particularly useful at creating a lookup table of callbacks or
/// individual channels for subscriptions, especially when the subscriptions
/// contain wildcards.
///
/// Note, however that there might be an issue with overlapped subscriptions
/// where callbacks are invoked multiple times for a message that matches
/// more than one subscription.
///
/// When using MQTT v5, subscription identifiers would be more efficient
/// and solve the problem of multiple overlapped callbacks. See:
/// <https://github.com/eclipse/paho.mqtt.rust/blob/master/examples/sync_consume_v5.rs>
///
#[derive(Debug)]
pub struct TopicMatcher<T> {
    root: Node<T>,
}

impl<T> TopicMatcher<T> {
    /// Creates a new, empty, topic matcher collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines if the collection contains no values.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Clears all the entries in the collection
    pub fn clear(&mut self) {
        self.root = Node::default();
    }

    /// Inserts a new topic filter into the collection.
    pub fn insert<S>(&mut self, filter: S, val: T)
    where
        S: Into<String>,
    {
        let filter = filter.into();
        let mut curr = &mut self.root;

        for field in filter.split('/') {
            curr = curr.children.entry(field.into()).or_default()
        }
        curr.value = Some((filter, val));
    }

    /// Gets a reference to a value from the collection using an exact
    /// filter match.
    pub fn get(&self, topic: &str) -> Option<&T> {
        let mut curr = &self.root;

        for field in topic.split('/') {
            curr = match curr.children.get(field) {
                Some(node) => node,
                None => return None,
            };
        }
        curr.value.as_ref().map(|(_, v)| v)
    }

    /// Gets a mutable mutable reference to a value from the collection
    /// using an exact filter match.
    pub fn get_mut(&mut self, topic: &str) -> Option<&mut T> {
        let mut curr = &mut self.root;

        for field in topic.split('/') {
            curr = match curr.children.get_mut(field) {
                Some(node) => node,
                None => return None,
            };
        }
        curr.value.as_mut().map(|(_, v)| v)
    }

    /// Gets an iterator over all the items in the collection.
    pub fn iter(&self) -> NodeIter<T> {
        self.root.iter()
    }

    /// Gets a muable iterator over all the items in the collection.
    pub fn iter_mut(&mut self) -> NodeIterMut<T> {
        self.root.iter_mut()
    }

    /// Gets an iterator for all the matches to the specified topic
    pub fn matches<'a, 'b>(&'a self, topic: &'b str) -> MatchIter<'a, 'b, T> {
        MatchIter::new(&self.root, topic)
    }

    /// Determines if the topic matches any of the filters in the collection.
    pub fn has_match(&self, topic: &str) -> bool {
        self.matches(topic).next().is_some()
    }
}

// We manually implement Default, otherwise the derived one would
// require T: Default.

impl<T> Default for TopicMatcher<T> {
    /// Create an empty TopicMatcher collection.
    fn default() -> Self {
        TopicMatcher {
            root: Node::default(),
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a TopicMatcher<T> {
    type Item = &'a (String, T);
    type IntoIter = NodeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut TopicMatcher<T> {
    type Item = &'a mut (String, T);
    type IntoIter = NodeIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/////////////////////////////////////////////////////////////////////////////

/// Iterator for the topic matcher collection.
/// This is created from a specific topic string and will find the contents
/// of all the matching filters in the collection.
/// Lifetimes:
///      'a - The matcher collection
///      'b - The topic string
///
/// We keep a stack of nodes that still need to be searched. For each node,
/// there is also an iterator stack of fields for that node to search.
#[derive(Debug)]
pub struct MatchIter<'a, 'b, T> {
    // The nodes still to be processed.
    // The tuple is (current node, remaining topic fields, if first node)
    remaining: Vec<(&'a Node<T>, Split<'b, char>, bool)>,
}

impl<'a, 'b, T> MatchIter<'a, 'b, T> {
    fn new(node: &'a Node<T>, topic: &'b str) -> Self {
        let fields = topic.split('/');
        Self {
            remaining: vec![(node, fields, true)],
        }
    }
}

impl<'a, 'b, T> Iterator for MatchIter<'a, 'b, T> {
    type Item = &'a (String, T);

    /// Gets the next value that matches the iterator's topic.
    fn next(&mut self) -> Option<Self::Item> {
        let (node, mut fields, first) = match self.remaining.pop() {
            Some(val) => val,
            None => return None,
        };

        let field = match fields.next() {
            Some(field) => field,
            None => return node.value.as_ref()
        };

        if let Some(child) = node.children.get(field) {
            self.remaining.push((child, fields.clone(), false));
        }

        // A topic starting with '$' doesn't match wildcards
        // https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901246

        if !first || !field.starts_with('$') {
            if let Some(child) = node.children.get("+") {
                self.remaining.push((child, fields, false))
            }

            if let Some(child) = node.children.get("#") {
                // By protocol definition, a '#' must be a terminating leaf.
                return child.value.as_ref();
            }
        }

        self.next()
    }
}

/// Macro to create a `TopicMatcher` collection.
#[macro_export]
macro_rules! topic_matcher {
    { $($filter:expr => $val:expr),+ } => {
        {
            let mut tm = crate::topic_matcher::TopicMatcher::new();
            $(
                tm.insert($filter, $val);
            )+
            tm
        }
     };
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_basic_topic_matcher() {
        let mut matcher: TopicMatcher<i32> = TopicMatcher::new();
        matcher.insert("some/test/topic", 19);

        assert_eq!(matcher.get("some/test/topic"), Some(&19));
        assert_eq!(matcher.get("some/test/bubba"), None);

        matcher.insert("some/+/topic", 42);
        matcher.insert("some/test/#", 99);
        matcher.insert("some/prod/topic", 155);

        assert!(matcher.has_match("some/random/topic"));
        assert!(!matcher.has_match("some/other/thing"));

        // Test the iterator

        let mut set = HashSet::new();
        set.insert(19);
        set.insert(42);
        set.insert(99);

        let mut match_set = HashSet::new();
        for (_k, v) in matcher.matches("some/test/topic") {
            match_set.insert(*v);
        }

        assert_eq!(set, match_set);
    }

    #[test]
    fn test_topic_matcher() {
        use crate::topic_matcher as tm;

        // Should match

        assert!(tm!{"foo/bar" => ()}.has_match("foo/bar"));
        assert!(tm!{"foo/+" => ()}.has_match("foo/bar"));
        assert!(tm!{"foo/+/baz" => ()}.has_match("foo/bar/baz"));
        assert!(tm!{"foo/+/#"=> ()}.has_match("foo/bar/baz"));
        assert!(tm!{"A/B/+/#"=> ()}.has_match("A/B/B/C"));
        assert!(tm!{"#"=> ()}.has_match("foo/bar/baz"));
        assert!(tm!{"#"=> ()}.has_match("/foo/bar"));
        assert!(tm!{"/#"=> ()}.has_match("/foo/bar"));
        assert!(tm!{"$SYS/bar"=> ()}.has_match("$SYS/bar"));
        assert!(tm!{"foo/#"=> ()}.has_match("foo/$bar"));
        assert!(tm!{"foo/+/baz"=> ()}.has_match("foo/$bar/baz"));

        // Should not match

        assert!(!tm!{"test/6/#"=> ()}.has_match("test/3"));
        assert!(!tm!{"foo/bar"=> ()}.has_match("foo"));
        assert!(!tm!{"foo/+"=> ()}.has_match("foo/bar/baz"));
        assert!(!tm!{"foo/+/baz"=> ()}.has_match("foo/bar/bar"));
        assert!(!tm!{"foo/+/#"=> ()}.has_match("fo2/bar/baz"));
        assert!(!tm!{"/#"=> ()}.has_match("foo/bar"));
        assert!(!tm!{"#"=> ()}.has_match("$SYS/bar"));
        assert!(!tm!{"$BOB/bar"=> ()}.has_match("$SYS/bar"));
        assert!(!tm!{"+/bar"=> ()}.has_match("$SYS/bar"));
    }

    #[test]
    fn test_topic_matcher_callback() {
        let mut matcher = TopicMatcher::new();

        matcher.insert("some/+/topic", Box::new(|n: u32| n * 2));

        for (_t, f) in matcher.matches("some/random/topic") {
            let n = f(2);
            assert_eq!(n, 4);
        }
    }

    #[test]
    fn assert_that_no_wildcards_matches() {
        assert!(topic_matches("a/b/c", "a/b/c"));
        assert!(topic_matches("foo/bar", "foo/bar"));
    }
    #[test]
    fn assert_that_plus_wildcard_matches() {
        assert!(topic_matches("a/+/c", "a/b/c"));
        assert!(topic_matches("foo/+/baz", "foo/bar/baz"));
    }
    #[test]
    fn assert_that_leading_plus_wildcard_matches() {
        assert!(topic_matches("+/b/c", "a/b/c"));
    }
    #[test]
    fn assert_that_trailing_plus_wildcard_matches() {
        assert!(topic_matches("a/b/+", "a/b/c"));
        assert!(topic_matches("foo/+", "foo/bar"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_none_level() {
        assert!(topic_matches("a/b/#", "a/b"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_single_level() {
        assert!(topic_matches("a/b/#", "a/b/c"));
    }
    #[test]
    fn assert_that_hash_wildcard_matches_multiple_levels() {
        assert!(topic_matches("a/b/#", "a/b/c/d"));
    }
    #[test]
    fn assert_that_single_hash_matches_all() {
        assert!(topic_matches("#", "foo/bar/baz"));
        assert!(topic_matches("#", "/foo/bar"));
        assert!(topic_matches("/#", "/foo/bar"));
    }
    #[test]
    fn assert_that_plus_and_hash_wildcards_matches() {
        assert!(topic_matches("foo/+/#", "foo/bar/baz"));
        assert!(topic_matches("A/B/+/#", "A/B/B/C"));
    }
    #[test]
    fn assert_that_sys_topic_matches() {
        assert!(topic_matches("$SYS/bar", "$SYS/bar"));
    }
    #[test]
    fn assert_that_non_first_levels_with_dollar_sign_matches_hash_wildcard() {
        assert!(topic_matches("foo/#", "foo/$bar"));
    }
    #[test]
    fn assert_that_non_first_levels_with_dollar_sign_matches_plus_wildcard() {
        assert!(topic_matches("foo/+/baz", "foo/$bar/baz"));
    }
    #[test]
    fn assert_that_different_levels_does_not_match() {
        assert!(!topic_matches("test/6/#", "test/3"));
        assert!(!topic_matches("foo/+/baz", "foo/bar/bar"));
        assert!(!topic_matches("foo/+/#", "fo2/bar/baz"));
        assert!(!topic_matches("$BOB/bar", "$SYS/bar"));
    }
    #[test]
    fn assert_that_longer_topics_does_not_match() {
        assert!(!topic_matches("foo/bar", "foo"));
    }
    #[test]
    fn assert_that_plus_wildcard_does_not_match_multiple_levels() {
        assert!(!topic_matches("foo/+", "foo/bar/baz"));
    }
    #[test]
    fn assert_that_leading_slash_with_hash_wildcard_does_not_match_normal_topic() {
        assert!(!topic_matches("/#", "foo/bar"));
    }
    #[test]
    fn assert_that_hash_wildcard_does_not_match_an_internal_topic() {
        assert!(!topic_matches("#", "$SYS/bar"));
    }
    #[test]
    fn assert_that_plus_wildcard_does_not_match_an_internal_topic() {
        assert!(!topic_matches("+/bar", "$SYS/bar"));
    }
}
