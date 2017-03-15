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

use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

///////////////////////
// Fixed Size BitSet //
///////////////////////

#[derive(Copy, Clone, PartialEq)]
pub struct BitSet32 {
    value: u32,
}

impl BitSet32 {
    pub fn new() -> Self {
        BitSet32 {
            value: 0,
        }
    }

    pub fn add(&mut self, i: u32) {
        self.value |= 0x80000000>>i
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, i: u32) {
        self.value &= !(0x80000000>>i)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.value = 0;
    }

    #[allow(dead_code)]
    pub fn set(&mut self, i: u32, v: bool) {
        match v {
            true => self.add(i),
            false => self.remove(i),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, i: u32) -> bool {
        self.value & 0x80000000>>i > 0
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> Iter32 {
        Iter32::new(self)
    }

    pub fn iter_ones(&self) -> IndexIter32 {
        IndexIter32::new(self)
    }

    #[allow(dead_code)]
    pub fn cardinality(&self) -> u32 {
        self.value.count_ones()
    }
}

impl Debug for BitSet32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "BitSet32({:032b})", self.value)
    }
}


pub struct Iter32<'a> {
    bitset: &'a BitSet32,
    i: u32,
}

impl<'a> Iter32<'a> {
    #[allow(dead_code)]
    fn new(bitset: &'a BitSet32) -> Self {
        Iter32 {
            bitset: bitset,
            i: 0,
        }
    }
}

impl<'a> Iterator for Iter32<'a> {
    type Item=bool;

    fn next(&mut self) -> Option<bool> {
        let i = self.i;

        if i < 32 {
            self.i += 1;
            Some(self.bitset.get(i))
        } else {
            None
        }
    }
}


pub struct IndexIter32<'a> {
    bitset: &'a BitSet32,
    i: u32,
}

impl<'a> IndexIter32<'a> {
    fn new(bitset: &'a BitSet32) -> Self {
        IndexIter32 {
            bitset: bitset,
            i: 0,
        }
    }
}

impl<'a> Iterator for IndexIter32<'a> {
    type Item=u32;

    fn next(&mut self) -> Option<u32> {
        let i = self.i;

        if i < 32 {
            let value = self.bitset.value & 0xFFFFFFFF>>i;
            if value > 0 {
                let lz = value.leading_zeros();
                self.i = lz+1;
                Some(lz)
            } else {
                self.i = 32;
                None
            }
        } else {
            None
        }
    }
}


#[cfg(test)]
mod test {
    use super::BitSet32;

    #[test]
    fn can_add() {
        let mut bs = BitSet32::new();

        assert_eq!(bs.get(0), false);
        bs.add(0);
        assert_eq!(bs.get(0), true);
    }

    #[test]
    fn can_remove() {
        let mut bs = BitSet32::new();

        bs.add(0);
        bs.add(1);
        bs.remove(1);
        assert_eq!(bs.get(0), true);
        assert_eq!(bs.get(1), false);
    }

    #[test]
    fn can_iter() {
        let mut bs = BitSet32::new();

        bs.add(0);
        bs.add(3);

        let mut iter = bs.iter();
        assert_eq!(iter.next(), Some(true));
        assert_eq!(iter.next(), Some(false));
        assert_eq!(iter.next(), Some(false));
        assert_eq!(iter.next(), Some(true));

        while let Some(b) = iter.next() {
            assert_eq!(b, false);
        }
    }

    #[test]
    fn can_iter_ones() {
        let mut bs = BitSet32::new();

        bs.add(0);

        {
            let mut iter = bs.iter_ones();
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), None);
        }

        bs.add(15);
        bs.add(31);

        {
            let mut iter = bs.iter_ones();
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(15));
            assert_eq!(iter.next(), Some(31));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn cardinality_is_correct() {
        let mut bs = BitSet32::new();

        assert_eq!(bs.cardinality(), 0);
        bs.add(0);
        assert_eq!(bs.cardinality(), 1);
        bs.add(15);
        assert_eq!(bs.cardinality(), 2);
        bs.add(31);
        assert_eq!(bs.cardinality(), 3);
        bs.remove(31);
        println!("{:?}", bs);
        assert_eq!(bs.cardinality(), 2);
    }

    #[test]
    fn format_is_correct() {
        let mut bs = BitSet32::new();

        assert_eq!(format!("{:?}", bs), "BitSet32(00000000000000000000000000000000)");
        bs.add(0);
        assert_eq!(format!("{:?}", bs), "BitSet32(10000000000000000000000000000000)");
    }
}