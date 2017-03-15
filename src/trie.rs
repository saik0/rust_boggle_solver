/* Copyright 2017 Joel Pedraza
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

/*
 * A Radix 26 Trie
 *
 * I'd prefer if if each letter was represented as Enum rather than a u8 (for safety)
 * Can they be used without sacrifing perf?
 */

use boggle_util;
use bitset::BitSet32;
use bitset::IndexIter32;

use std::mem;

type Node = Option<Box<Trie>>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeType {
    Prefix,
    Word(usize),
}

#[derive(Debug)]
pub struct Trie {
    node_type: NodeType,
    children: [Node; boggle_util::ALPHABET_SIZE],
    child_set: BitSet32,
}

impl Trie {
	pub fn new() -> Self {
        Trie {
            node_type: NodeType::Prefix,
            children: Default::default(),
            child_set: BitSet32::new(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn insert(&mut self, s: &str, id: usize) -> bool {
    	if boggle_util::is_alpha(s) {
    		self.ins(s.to_lowercase().as_bytes(), id);
            true
    	} else {
            false
        }
    }

    #[inline]
    fn ins(&mut self, s: &[u8], id: usize) -> () {
        let first = boggle_util::ascii_byte_to_idx(s[0]);

        if self.children[first].is_none() {
            self.child_set.add(first as u32);
            mem::replace(&mut (self.children[first]), Some(Box::new(Trie::new())));
        }

        let child = self.children[first].as_mut().unwrap();

        if s.len() > 1 {
            child.ins(&s[1..], id);
        } else {
            child.node_type = NodeType::Word(id);
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, s: &str) -> Option<NodeType> {
        if boggle_util::is_alpha(s) {
            self.cns(s.to_lowercase().as_bytes())
        } else {
            None
        }
    }

    #[inline]
    fn cns(&self, s: &[u8]) -> Option<NodeType> {
        let first = boggle_util::ascii_byte_to_idx(s[0]);

        if let Some(child) = self.children[first].as_ref() {
            if s.len() == 1 {
                Some(child.node_type)
            } else {
                let rest = &s[1..];
                child.cns(rest)
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> TrieIterator {
        TrieIterator::new(self)
    }
}


pub struct TrieIterator<'a> {
    trie: &'a Trie,
    iter: IndexIter32<'a>,
}

impl<'a> TrieIterator<'a> {
    fn new(trie: &'a Trie) -> TrieIterator<'a> {
        TrieIterator {
            trie: trie,
            iter: trie.child_set.iter_ones(),
        }
    }
}

impl<'a> Iterator for TrieIterator<'a> {
    type Item = (&'a Trie, u8);

    fn next(&mut self) -> Option<(&'a Trie, u8)> {
        match self.iter.next() {
            Some(i) => {
                match self.trie.children[i as usize] {
                    Some(ref trie) => Some((trie, i as u8)),
                    None => None
                }
            },
            None => None
        }
    }
}


//==============================================================================


#[cfg(test)]
mod test{

    use std::str;
    use super::Trie;
    use super::NodeType;

    #[test]
    fn valid_words_are_inserted() {
        let mut trie = Trie::new();

        assert_eq!(trie.contains("a"), None);
        assert_eq!(trie.contains("abba"), None);

        assert!(trie.insert("abba", 0));

        assert_eq!(trie.contains("a"), Some(NodeType::Prefix));
        assert_eq!(trie.contains("ab"), Some(NodeType::Prefix));
        assert_eq!(trie.contains("abb"), Some(NodeType::Prefix));
        assert_eq!(trie.contains("abba"), Some(NodeType::Word(0)));
    }

    #[test]
    fn invalid_words_are_not_inserted() {
        let mut trie = Trie::new();

        let mut id = 0;
        for s in ('\u{0}' as u8 .. 'A' as u8)
                 .chain('[' as u8 .. 'a' as u8)
                 .chain('{' as u8 .. '\u{ff}' as u8)
                 .map(|b| unsafe { str::from_utf8_unchecked(&[b]) }.to_owned() ) {
            id += 1;
            assert!(!trie.insert(&s, id));
            assert_eq!(trie.contains(&s), None);
        }
    }

    #[test]
    fn is_case_insensitive() {
        let mut trie = Trie::new();

        trie.insert("a", 0);
        assert_eq!(trie.contains("a"), Some(NodeType::Word(0)));
        assert_eq!(trie.contains("A"), Some(NodeType::Word(0)));

        trie.insert("B", 1);
        assert_eq!(trie.contains("b"), Some(NodeType::Word(1)));
        assert_eq!(trie.contains("B"), Some(NodeType::Word(1)));
    }
}