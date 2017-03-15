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

use std;

type SimpleBoggleCell = u8;

#[derive(Debug)]
pub struct SimpleBoggleBoard {
    width: usize,
    height: usize,
    cells: Box<[SimpleBoggleCell]>,
}

impl SimpleBoggleBoard {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        SimpleBoggleBoard {
            width: width,
            height: height,
            cells: vec![ Default::default(); width * height ].into_boxed_slice(),
        }
    }

    pub fn read<'a, I>(mut lines: I) -> Result<SimpleBoggleBoard, &'static str> where I: Iterator<Item=&'a str> {
        if let Some(first) = lines.next() {
            let first = &first.trim().to_lowercase();
            if !boggle_util::is_alpha(first) {
                return Err("Invalid chars");
            }

            let width = first.len();
            let mut height = 1;
            let mut cells: Vec<SimpleBoggleCell> = Vec::new();
            
            cells.extend(first.as_bytes().iter().map(|b| b - 'a' as u8));

            while let Some(line) = lines.next() {
                let line = &line.trim().to_lowercase();
                if !boggle_util::is_alpha(first) {
                    return Err("Invalid chars");
                }

                if line.len() != width {
                    return Err("Invalid line length");
                }

                cells.extend(line.as_bytes().iter().map(|b| b - 'a' as u8));

                height += 1;
            }

            if height > 1 {
                Ok(SimpleBoggleBoard{
                    width: width,
                    height: height,
                    cells: cells.into_boxed_slice()
                })
            } else {
                Err("Board height must be >= 2")
            }
        } else {
            Err("Empty iterator")
        }
        
        
    }

    #[allow(dead_code)]
    pub fn set(&mut self, i: usize, v: SimpleBoggleCell) -> () {
        self.cells[i] = v;
    }

    // TODO REMOVE
    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.cells.iter()
    }
}



impl /* BoggleBoard for*/ SimpleBoggleBoard {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn neighbors(&self, i: usize, v:SimpleBoggleCell) -> std::vec::IntoIter<usize> {
        let cands: Box<[(isize, isize)]> = match i {
            0 => Box::new([( 1isize,  0isize),
                           ( 1isize,  1isize),
                           ( 0isize,  1isize)]),

            x if x == self.width -1
              => Box::new([( 0isize,  1isize),
                           (-1isize,  1isize),
                           (-1isize,  0isize)]),

            x if x == self.width * (self.height - 1)
              => Box::new([( 0isize, -1isize),
                           ( 1isize, -1isize),
                           ( 1isize,  0isize)]),

            x if x == self.width * self.height - 1
              => Box::new([( 0isize, -1isize),
                           (-1isize,  0isize),
                           (-1isize, -1isize)]),

            x if x < self.width
              => Box::new([( 1isize,  0isize),
                           ( 1isize,  1isize),
                           ( 0isize,  1isize),
                           (-1isize,  1isize),
                           (-1isize,  0isize)]),

            x if x > self.width * (self.height - 1)
              => Box::new([( 0isize, -1isize),
                           ( 1isize, -1isize),
                           ( 1isize,  0isize),
                           (-1isize,  0isize),
                           (-1isize, -1isize)]),

            x if x % self.width == 0
              => Box::new([( 0isize, -1isize),
                           ( 1isize, -1isize),
                           ( 1isize,  0isize),
                           ( 1isize,  1isize),
                           ( 0isize,  1isize)]),

            x if x % self.width == self.width - 1
              => Box::new([( 0isize, -1isize),
                           ( 0isize,  1isize),
                           (-1isize,  1isize),
                           (-1isize,  0isize),
                           (-1isize, -1isize)]),

            _ => Box::new([( 0isize, -1isize),
                  ( 1isize, -1isize),
                  ( 1isize,  0isize),
                  ( 1isize,  1isize),
                  ( 0isize,  1isize),
                  (-1isize,  1isize),
                  (-1isize,  0isize),
                  (-1isize, -1isize)])
        };

        let v: Vec<usize> = cands
            .iter()
            .map(|&coord| (coord.0 + self.width as isize * coord.1) as isize)
            .map(|rel_idx| (i as isize + rel_idx) as usize)
            .filter(|abs_idx| self.cells[*abs_idx] == v)
            .collect();
        v.into_iter()
    }

    pub fn any(&self, v:SimpleBoggleCell) -> std::vec::IntoIter<usize> {
        let v: Vec<usize> = self.cells
            .iter()
            .enumerate()
            .filter(|&(_, &x)| x == v)
            .map(&|(i, &_)| i)
            .collect();
        v.into_iter()
    }
}