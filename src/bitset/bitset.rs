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

use std::cmp::max;

const TWO_POW_64: u64 = 0x8000000000000000;
const MAX: u64        = 0xFFFFFFFFFFFFFFFF;

#[derive(Debug, Clone)]
pub struct BitSet {
    data: Vec<u64>,
    len: usize
}

impl BitSet {
    pub fn new() -> Self {
        BitSet {
            data: Vec::new(),
            len: 0
        }
    }

    #[inline]
    fn idx(i: usize) -> (usize, u32) {
        (i / 64, (i % 64) as u32)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, i: usize) -> bool {
        let ref data = self.data;
        let (idx, off) = Self::idx(i);

        if data.len() > idx {
            unsafe { self.get_unsafe(idx, off) }
        } else {
            false
        }
    }

    #[inline]
    unsafe fn get_unsafe(&self, idx: usize, off: u32) -> bool {
        self.data.get_unchecked(idx) & TWO_POW_64 >> off > 0
    }

    pub fn add(&mut self, i: usize) {
        let ref mut data = self.data;
        let (idx, off) = Self::idx(i);

        if idx >= data.len() {
            data.resize(idx + 1, 0);
        }

        self.len = max(self.len, i+1);
        // Guaranteed to be in bounds, as we have expanded data
        unsafe { *data.get_unchecked_mut(idx) |= TWO_POW_64 >> off };
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, i: usize) {
        if i < self.len() {
            let ref mut data = self.data;
            let (idx, off) = Self::idx(i);

            // i < self.len() guarantees in bounds 
            unsafe { *data.get_unchecked_mut(idx) &= !(TWO_POW_64 >> off) };

            // The highest set bit was unset, find the next highest and set self.len to it's index
            if i == self.len - 1 {
                for (idx, datum) in data.iter().enumerate().rev() {
                    if *datum > 0 {
                        self.len = idx * 64 + (64 - datum.trailing_zeros() as usize);
                        return;
                    }
                }

                self.len = 0;
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        let ref mut data = self.data;

        self.len = 0;
        for datum in data.iter_mut() {
            *datum = 0;
        }
    }

    pub fn iter_ones(&self) -> IndexIter {
        IndexIter::new(self)
    }
}

#[allow(dead_code)]
pub struct Iter<'a> {
    bitset: &'a BitSet,
    idx: usize,
    off: u32,
}

impl<'a> Iter<'a> {
    #[allow(dead_code)]
    fn new(bitset: &'a BitSet) -> Self {
        Iter {
            bitset: bitset,
            idx: 0,
            off: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item=bool;

    fn next(&mut self) -> Option<bool> {
        if (self.idx * 64 + self.off as usize) < self.bitset.len() {
            let result = unsafe { self.bitset.get_unsafe(self.idx, self.off) };
            if self.off < 64 {
                self.off += 1;
            } else {
                self.idx +=1;
                self.off = 0;
            }
            Some(result)
        } else {
            None
        }
    }
}

// =============================================================================

pub struct IndexIter<'a> {
    bitset: &'a BitSet,
    idx: usize,
    off: u32,
}

impl<'a> IndexIter<'a> {
    fn new(bitset: &'a BitSet) -> Self {
        IndexIter {
            bitset: bitset,
            idx: 0,
            off: 0,
        }
    }
}

impl<'a> Iterator for IndexIter<'a> {
    type Item=usize;

    fn next(&mut self) -> Option<usize> {
        let ref data = self.bitset.data;

        while self.idx < data.len() {
            match unsafe { data.get_unchecked(self.idx) } & MAX >> self.off {
                0 => {
                    self.idx += 1;
                    self.off  = 0;
                    continue;
                },
                1 => {
                    self.idx += 1;
                    self.off  = 0;
                    return Some(self.idx * 64 - 1)
                },
                v => {
                    let lz = v.leading_zeros();
                    self.off = lz+1;
                    return Some(self.idx * 64 + lz as usize);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::BitSet;

    #[test]
    fn can_add() {
        let mut bs = BitSet::new();

        assert_eq!(bs.get(0), false);
        bs.add(0);
        assert_eq!(bs.get(0), true);

        assert_eq!(bs.get(256), false);
        bs.add(256);
        assert_eq!(bs.get(256), true);
    }

    #[test]
    fn can_remove() {
        let mut bs = BitSet::new();

        bs.add(0);
        bs.remove(0);
        assert_eq!(bs.get(0), false);
    }

    #[test]
    fn len_is_correct() {
        let mut bs = BitSet::new();

        assert_eq!(bs.len(), 0);

        bs.add(0);
        assert_eq!(bs.len(), 1);

        bs.remove(0);
        assert_eq!(bs.len(), 0);

        bs.add(130);
        assert_eq!(bs.len(), 131);

        bs.add(7);
        assert_eq!(bs.len(), 131);

        bs.remove(130);
        assert_eq!(bs.len(), 8);
    }

    #[test]
    fn can_iter_indices() {
        let mut bs = BitSet::new();

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

        bs.remove(31);
        bs.add(127);
        bs.add(587);

        {
            let mut iter = bs.iter_ones();
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(15));
            assert_eq!(iter.next(), Some(127));
            assert_eq!(iter.next(), Some(587));
            assert_eq!(iter.next(), None);
        }
    }
}
