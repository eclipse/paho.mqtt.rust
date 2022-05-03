// topic_matcher.rs
//
// A collection for matching a topic to a set of filters.
//
// This file is part of the Eclipse Paho MQTT Rust Client library.
//

/*******************************************************************************
 * Copyright (c) 2021-2022 Frank Pagliughi <fpagliughi@mindspring.com>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

//! Code to match MQTT topics to filters that may contain wildcards.
//!

use std::collections::HashMap;

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

/// A collection of topic filters that can compare against a topic and
/// produce an iterator of all matched items.
///
/// This is particularly useful at creating a lookup table of callbacks
/// for subscriptions, especially when the subscriptions contain wildcards.
/// Note, however that there might be an issue with overlapped subscription
/// where callbacks are invoked multiple times for a messgag that matches
/// more than one subscription.
///
/// When using MQTT v5, subscription identifiers would be more efficient,
/// and solve the problem of multiple overlapped callbacks. See:
/// <https://github.com/eclipse/paho.mqtt.rust/blob/master/examples/sync_consume_v5.rs>
///
pub struct TopicMatcher<T> {
    /// The root node of the collection.
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

    /// Inserts a new topic filter into the collection.
    pub fn insert<S>(&mut self, key: S, val: T)
    where
        S: Into<String>,
    {
        let key = key.into();
        let mut node = &mut self.root;

        for sym in key.split('/') {
            node = node
                .children
                .entry(sym.to_string())
                .or_insert_with(Node::<T>::default);
        }
        // We've either found or created nodes down to here.
        node.content = Some((key, val));
    }

    /// Gets a value from the collection using an exact filter match.
    pub fn get(&self, key: &str) -> Option<&T> {
        let mut node = &self.root;
        for sym in key.split('/') {
            node = match node.children.get(sym) {
                Some(node) => node,
                None => return None,
            }
        }
        node.content.as_ref().map(|(_,v)| v)
    }

    /// Gets an iterator for all the matches to the specified
    pub fn matches<'a, 'b>(&'a self, topic: &'b str) -> MatchIter<'a, 'b, T> {
        let syms: Vec<_> = topic.split('/').collect();
        MatchIter {
            node: Some(&self.root),
            syms,
            nodes: Vec::new(),
        }
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

/// A single node in the topic matcher collection.
struct Node<T> {
    /// The value that matches the topic at this node, if any.
    content: Option<(String, T)>,
    /// The child nodes mapped by the next field of the topic.
    children: HashMap<String, Node<T>>,
}

impl<T> Node<T> {
    /// Determines if the node does not contain a value.
    ///
    /// This is a relatively simplistic implementation indicating that the
    /// node's content and children are empty. Technically, the node could
    /// contain a collection of children that are empty, which might be
    /// considered an "empty" state. But not here.
    fn is_empty(&self) -> bool {
        self.content.is_none() && self.children.is_empty()
    }
}

// We manually implement Default, otherwise the derived one would
// require T: Default.

impl<T> Default for Node<T> {
    /// Creates a default, empty node.
    fn default() -> Self {
        Node {
            content: None,
            children: HashMap::new(),
        }
    }
}

/// Iterator for the topic matcher collection.
/// This is created from a specific topic string and will find the contents
/// of all the matching filters in the collection.
/// Lifetimes:
///      'a - The matcher collection
///      'b - The original topic string
pub struct MatchIter<'a, 'b, T> {
    /// The current node to search
    node: Option<&'a Node<T>>,
    // The topic we're searching on, split into fields
    syms: Vec<&'b str>,
    // The nodes still to be processed
    nodes: Vec<(&'a Node<T>, Vec<&'b str>)>,
}

impl<'a, 'b, T> Iterator for MatchIter<'a, 'b, T> {
    type Item = &'a (String, T);

    /// Gets the next value from a key filter that matches the iterator's topic.
    fn next(&mut self) -> Option<Self::Item> {
        let node = match self.node.take() {
            Some(node) => node,
            None => return None,
        };
        let mut c = None;

        if self.syms.is_empty() {
            c = node.content.as_ref();
        }
        else {
            if let Some(child) = node.children.get(self.syms[0]) {
                let syms = self.syms[1..].to_vec();
                self.nodes.push((child, syms));
            }
            if let Some(child) = node.children.get("+") {
                let syms = self.syms[1..].to_vec();
                self.nodes.push((child, syms))
            }
            if let Some(child) = node.children.get("#") {
                c = child.content.as_ref();
            }
        }

        if let Some((child, syms)) = self.nodes.pop() {
            self.node = Some(child);
            self.syms = syms;
            c.or_else(|| self.next())
        }
        else {
            c
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
//                              Unit Tests
/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_topic_matcher() {
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
    fn test_topic_matcher_callback() {
        let mut matcher = TopicMatcher::new();

        matcher.insert("some/+/topic", Box::new(|n: u32| {
            n * 2
        }));

        for (t, f) in matcher.matches("some/random/topic") {
            let n = f(2);
            assert_eq!(n, 4);
        }
    }
}
