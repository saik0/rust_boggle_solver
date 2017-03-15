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

use boggle_util;
use bitset::BitSet;
use bitset::IndexIter;

use super::SimpleBoggleBoard;

/*
 * Can this be done cleaner with Enums and some sort of EnumSet
 */

const FLAG_NORTHWEST : u8  = 0b10000000;
const FLAG_NORTH     : u8  = 0b01000000;
const FLAG_NORTHEAST : u8  = 0b00100000;
const FLAG_WEST      : u8  = 0b00010000;
const FLAG_EAST      : u8  = 0b00001000;
const FLAG_SOUTHWEST : u8  = 0b00000100;
const FLAG_SOUTH     : u8  = 0b00000010;
const FLAG_SOUTHEAST : u8  = 0b00000001;


type RadixBoggleCell = [u8; boggle_util::ALPHABET_SIZE];

pub struct RadixBoggleBoard {
    width: usize,
    height: usize,
    /// Top level navigation by value
    /// (Used to quickly find all the cells on the board of a specified value)
    alpha: [BitSet; boggle_util::ALPHABET_SIZE],
    /// Serves as a precomupted adjacency matrix filtered by value
    /// (Used to quickly find all the neighbors of a cell of a specified value)
    cells: Box<[RadixBoggleCell]>,
}

impl RadixBoggleBoard {
    pub fn new(width: usize, height: usize) -> Self {
        use std::mem;
        use std::ptr;

        let mut alpha: [BitSet; boggle_util::ALPHABET_SIZE];
        unsafe {
            alpha = mem::uninitialized();

            for element in alpha.iter_mut() {
                let bs = BitSet::new();
                ptr::write(element, bs);
            }
        }

        RadixBoggleBoard {
            width: width,
            height: height,
            alpha: alpha,
            cells: vec![ Default::default(); width * height ].into_boxed_slice(),
        }
    }

    /// Creates a new radix board from a filled simple board
    /// this is mostly due to laziness, as SimpleBoggleBoard::read
    /// could be cleaner and I dont want to duplicate ugly code xD
    /// Maybe use a generified builder?
    pub fn from(src: &SimpleBoggleBoard) -> Self {
        let mut dst = Self::new(src.width(), src.height());
        for (i, v) in src.iter().enumerate() {
            dst.set(i, *v);
        }
        dst
    }

    pub fn any(&self, v: u8) -> IndexIter {
        self.alpha[v as usize].iter_ones()
    }

    pub fn neighbors(&self, i: usize, v: u8) -> RadixNeighborIter {
        RadixNeighborIter {
            v: self.cells[i][v as usize],
            i: i,
            w: self.width
        }
    }

    #[inline]
    fn mask_cell(&mut self, v: usize, i: usize, mask: u8) {
        self.cells[i][v] |= mask;
    }


    pub fn set(&mut self, i: usize, v: u8) {
        self.alpha[v as usize].add(i);
        
        let w = self.width;
        let v = v as usize;

        // Mask each neighbor of i with the flag for v (relative to i)
        match i {
            // northwest corner
            0 => {
                self.mask_cell(v, i  +1, FLAG_WEST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
                self.mask_cell(v, i+w+1, FLAG_NORTHWEST);
            },

            // northeast corner
            x if x == self.width -1 => {
                self.mask_cell(v, i  -1, FLAG_EAST);
                self.mask_cell(v, i+w-1, FLAG_NORTHEAST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
            },

            // southwest corner
            x if x == self.width * (self.height - 1) => {
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i-w+1, FLAG_SOUTHWEST);
                self.mask_cell(v, i  +1, FLAG_WEST);
            },

            // southeast corner
            x if x == self.width * self.height - 1 => {
                self.mask_cell(v, i-w-1, FLAG_SOUTHEAST);
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i  -1, FLAG_EAST);
            },

            // north edge
            x if x < self.width => {
                self.mask_cell(v, i  -1, FLAG_EAST);
                self.mask_cell(v, i  +1, FLAG_WEST);
                self.mask_cell(v, i+w-1, FLAG_NORTHEAST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
                self.mask_cell(v, i+w+1, FLAG_NORTHWEST);
            },

            // south edge
            x if x > self.width * (self.height - 1) => {
                self.mask_cell(v, i-w-1, FLAG_SOUTHEAST);
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i-w+1, FLAG_SOUTHWEST);
                self.mask_cell(v, i  -1, FLAG_EAST);
                self.mask_cell(v, i  +1, FLAG_WEST);
            },

            // west edge
            x if x % self.width == 0 => {
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i-w+1, FLAG_SOUTHWEST);
                self.mask_cell(v, i  +1, FLAG_WEST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
                self.mask_cell(v, i+w+1, FLAG_NORTHWEST);
            },

            // east edge
            x if x % self.width == self.width - 1 => {
                self.mask_cell(v, i-w-1, FLAG_SOUTHEAST);
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i  -1, FLAG_EAST);
                self.mask_cell(v, i+w-1, FLAG_NORTHEAST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
            },

            // interior
            _ => {
                self.mask_cell(v, i-w-1, FLAG_SOUTHEAST);
                self.mask_cell(v, i-w  , FLAG_SOUTH);
                self.mask_cell(v, i-w+1, FLAG_SOUTHWEST);
                self.mask_cell(v, i  -1, FLAG_EAST);
                self.mask_cell(v, i  +1, FLAG_WEST);
                self.mask_cell(v, i+w-1, FLAG_NORTHEAST);
                self.mask_cell(v, i+w  , FLAG_NORTH);
                self.mask_cell(v, i+w+1, FLAG_NORTHWEST);
            }
        }
    }
}

pub struct RadixNeighborIter {
    value: u8,
    // the index of the cell
    idx: usize,
    // the width of the board
    width: usize,
}

impl Iterator for RadixNeighborIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self.value.leading_zeros() {
            0 => { self.value &=0b01111111; Some(self.idx - self.width - 1) }
            1 => { self.value &=0b00111111; Some(self.idx - self.width    ) },
            2 => { self.value &=0b00011111; Some(self.idx - self.width + 1) },
            3 => { self.value &=0b00001111; Some(self.idx              - 1) },
            4 => { self.value &=0b00000111; Some(self.idx              + 1) },
            5 => { self.value &=0b00000011; Some(self.idx + self.width - 1) },
            6 => { self.value &=0b00000001; Some(self.idx + self.width    ) },
            7 => { self.value  =0b00000000; Some(self.idx + self.width + 1) },
            _ => None
        }
    }
}
