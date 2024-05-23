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
 * are made available under the terms of the Eclipse Public License v2.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v20.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

// TODO: Remove for production
#![allow(dead_code)]

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

    /// Inserts a new topic filter into the collection.
    pub fn insert<S>(&mut self, key: S, val: T)
    where
        S: Into<String>,
    {
        let key = key.into();
        let mut node = &mut self.root;

        for sym in key.split('/') {
            node = match sym {
                "+" => node.plus_wild.get_or_insert(Box::<Node<T>>::default()),
                "#" => node.pound_wild.get_or_insert(Box::<Node<T>>::default()),
                sym => node.children.entry(sym.to_string()).or_default(),
            }
        }
        // We've either found or created nodes down to here.
        node.content = Some((key, val));
    }

    /// Removes the entry, returning the value for it, if found.
    pub fn remove(&mut self, key: &str) -> Option<T> {
        // TODO: If the node is empty after removing the item, we should
        //   remove the node and all empty nodes above it.
        let mut node = &mut self.root;
        for sym in key.split('/') {
            let node_opt = match sym {
                "+" => node.plus_wild.as_deref_mut(),
                "#" => node.pound_wild.as_deref_mut(),
                sym => node.children.get_mut(sym),
            };
            node = match node_opt {
                Some(node) => node,
                None => return None,
            };
        }
        node.content.take().map(|(_, v)| v)
    }

    /// Gets a reference to a value from the collection using an exact
    /// filter match.
    pub fn get(&self, key: &str) -> Option<&T> {
        let mut node = &self.root;
        for sym in key.split('/') {
            let node_opt = match sym {
                "+" => node.plus_wild.as_deref(),
                "#" => node.pound_wild.as_deref(),
                sym => node.children.get(sym),
            };
            node = match node_opt {
                Some(node) => node,
                None => return None,
            };
        }
        node.content.as_ref().map(|(_, v)| v)
    }

    /// Gets a mutable mutable reference to a value from the collection
    /// using an exact filter match.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        let mut node = &mut self.root;
        for sym in key.split('/') {
            let node_opt = match sym {
                "+" => node.plus_wild.as_deref_mut(),
                "#" => node.pound_wild.as_deref_mut(),
                sym => node.children.get_mut(sym),
            };
            node = match node_opt {
                Some(node) => node,
                None => return None,
            };
        }
        node.content.as_mut().map(|(_, v)| v)
    }

    /// Gets an iterator for all the matches to the specified topic
    pub fn matches<'a, 'b>(&'a self, topic: &'b str) -> MatchIter<'a, 'b, T> {
        MatchIter::new(&self.root, topic)
    }

    /// Gets a mutable iterator for all the matches to the specified topic
    pub fn matches_mut<'a, 'b>(&'a mut self, topic: &'b str) -> MatchIterMut<'a, 'b, T> {
        MatchIterMut::new(&mut self.root, topic)
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
struct Node<T> {
    /// The value that matches the topic at this node, if any.
    content: Option<(String, T)>,
    /// The explicit, non-wildcardchild nodes mapped by the next field of
    /// the topic.
    children: HashMap<String, Node<T>>,
    /// Matches against the '+' wildcard
    plus_wild: Option<Box<Node<T>>>,
    /// Matches against the (terminating) '#' wildcard
    /// TODO: This is a terminating leaf. We can insert just a value,
    ///   instad of a Node.
    pound_wild: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    /// Determines if the node does not contain a value.
    ///
    /// This is a relatively simplistic implementation indicating that the
    /// node's content and children are empty. Technically, the node could
    /// contain a collection of children that are empty, which might be
    /// considered an "empty" state. But not here.
    fn is_empty(&self) -> bool {
        self.content.is_none()
            && self.children.is_empty()
            && self.plus_wild.is_none()
            && self.pound_wild.is_none()
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
            plus_wild: None,
            pound_wild: None,
        }
    }
}

/// Iterator for the topic matcher collection.
/// This is created from a specific topic string and will find the contents
/// of all the matching filters in the collection.
/// Lifetimes:
///      'a - The matcher collection
///      'b - The original topic string
///
/// We keep a stack of nodes that still need to be searched. For each node,
/// there is also a stack of keys for that node to search. The keys are kept
/// in reverse order, where the next ket to be searched can be popped off the
/// back of the vector.
pub struct MatchIter<'a, 'b, T> {
    // The nodes still to be processed
    nodes: Vec<(&'a Node<T>, Vec<&'b str>)>,
}

impl<'a, 'b, T> MatchIter<'a, 'b, T> {
    fn new(node: &'a Node<T>, topic: &'b str) -> Self {
        let syms: Vec<_> = topic.rsplit('/').collect();
        Self {
            nodes: vec![(node, syms)],
        }
    }
}

impl<'a, 'b, T> Iterator for MatchIter<'a, 'b, T> {
    type Item = &'a (String, T);

    /// Gets the next value from a key filter that matches the iterator's topic.
    fn next(&mut self) -> Option<Self::Item> {
        let (node, mut syms) = match self.nodes.pop() {
            Some(val) => val,
            None => return None,
        };

        let sym = match syms.pop() {
            Some(sym) => sym,
            None => return node.content.as_ref(),
        };

        if let Some(child) = node.children.get(sym) {
            self.nodes.push((child, syms.clone()));
        }

        if let Some(child) = node.plus_wild.as_ref() {
            self.nodes.push((child, syms))
        }

        if let Some(child) = node.pound_wild.as_ref() {
            // By protocol definition, a '#' must be a terminating leaf.
            return child.content.as_ref();
        }

        self.next()
    }
}

/// Mutable iterator for the topic matcher collection.
/// This is created from a specific topic string and will find the contents
/// of all the matching filters in the collection.
/// Lifetimes:
///      'a - The matcher collection
///      'b - The original topic string
pub struct MatchIterMut<'a, 'b, T> {
    // The nodes still to be processed
    nodes: Vec<(&'a mut Node<T>, Vec<&'b str>)>,
}

impl<'a, 'b, T> MatchIterMut<'a, 'b, T> {
    fn new(node: &'a mut Node<T>, topic: &'b str) -> Self {
        let syms: Vec<_> = topic.rsplit('/').collect();
        Self {
            nodes: vec![(node, syms)],
        }
    }
}

impl<'a, 'b, T> Iterator for MatchIterMut<'a, 'b, T> {
    type Item = (&'a String, &'a mut T);

    /// Gets the next value from a key filter that matches the iterator's topic.
    fn next(&mut self) -> Option<Self::Item> {
        let (node, mut syms) = match self.nodes.pop() {
            Some(val) => val,
            None => return None,
        };

        let sym = match syms.pop() {
            Some(sym) => sym,
            None => return node.content.as_mut().map(|(k, v)| (&*k, v)),
        };

        if let Some(child) = node.children.get_mut(sym) {
            self.nodes.push((child, syms.clone()));
        }

        if let Some(child) = node.plus_wild.as_mut() {
            self.nodes.push((child, syms))
        }

        if let Some(child) = node.pound_wild.as_mut() {
            // By protocol definition, a '#' must be a terminating leaf.
            return child.content.as_mut().map(|(k, v)| (&*k, v));
        }

        self.next()
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

        matcher.insert("some/+/topic", Box::new(|n: u32| n * 2));

        for (_t, f) in matcher.matches("some/random/topic") {
            let n = f(2);
            assert_eq!(n, 4);
        }
    }
}
