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

mod boggle_util;
mod bitset;
mod trie;
mod boggle;

extern crate rayon;

use trie::NodeType;
use trie::Trie;
use boggle::*;
use bitset::BitSet;

use rayon::prelude::*;

/*
 * Recursively search for dictionary words on the boggle board
 *
 * By using a prefix trie we prune words that cant be found
 * (because their prefixes aren't on the board)
 */

#[allow(dead_code)]
fn solve(root: &Trie, board: &SimpleBoggleBoard) {
    let mut word = Vec::with_capacity(64);
    let mut path = Vec::with_capacity(64);
    let mut words = BitSet::new();
    for (trie, i) in root.iter() {
        word.push(i);
        for pos in board.any(i) {
            path.push(pos);
            descend(trie, board, &mut word, &mut path, &mut words);
            path.pop();
        }
        word.pop();
    }
}

#[allow(dead_code)]
fn par_solve(root: &Trie, board: &SimpleBoggleBoard) {

    let forest: Vec<(&Trie, u8)> = root.iter().collect();
    forest.par_iter().for_each(|&(ref trie, i)| {
        let mut word = Vec::with_capacity(64);
        let mut path = Vec::with_capacity(64);
        let mut words = BitSet::new();

        word.push(i);
        for pos in board.any(i) {
            path.push(pos);
            descend(trie, board, &mut word, &mut path, &mut words);
            path.pop();
        }
    });
}

/*
 * Keeps track of found words with a bitset keyed by word id.
 * Removing the word fom the trie would probably reduce the searching,
 * I'll look into it.
 */

#[inline]
#[allow(dead_code)]
fn descend(parent: &Trie, board: &SimpleBoggleBoard, mut word: &mut Vec<u8>, mut path: &mut Vec<usize>, mut words: &mut BitSet) {
    for (trie, i) in parent.iter() {
        word.push(i);

        for pos in board.neighbors(*path.last().unwrap(), i) {

            if !path.contains(&pos) {
                path.push(pos);

                match trie.node_type() {
                    NodeType::Word(id) => {
                        if !words.get(id) {
                            words.add(id);
                            let mut found: Vec<u8> = word.clone();
                            for b in found.iter_mut() {
                                *b += 'a' as u8;
                            }

                            let s = unsafe { std::str::from_utf8_unchecked(&found) }.replace("q", "qu");
                            println!("{}", s);
                        }
                    },
                    _ => ()
                }


                descend(trie, board, &mut word, &mut path, &mut words);
                path.pop();
            }
        }
        word.pop();
    }
}


//==============================================================================


/*
 * Here we have a the same funtion body, but with different signatures.
 * I'm not sure how to make the two board types impl the BoggleBoard trait without
 * boxing, which slows down the algo. (not even in nightly via impl trait)
 */

#[allow(dead_code)]
fn solve_radix(root: &Trie, board: &RadixBoggleBoard) {
    let mut word = Vec::with_capacity(64);
    let mut path = Vec::with_capacity(64);
    let mut words = BitSet::new();
    for (trie, i) in root.iter() {
        word.push(i);
        for pos in board.any(i) {
            path.push(pos);
            descend_radix(trie, board, &mut word, &mut path, &mut words);
            path.pop();
        }
        word.pop();
    }
}

#[allow(dead_code)]
fn par_solve_radix(root: &Trie, board: &RadixBoggleBoard) {

    let forest: Vec<(&Trie, u8)> = root.iter().collect();
    forest.par_iter().for_each(|&(ref trie, i)| {
        let mut word = Vec::with_capacity(64);
        let mut path = Vec::with_capacity(64);
        let mut words = BitSet::new();

        word.push(i);
        for pos in board.any(i) {
            path.push(pos);
            descend_radix(trie, board, &mut word, &mut path, &mut words);
            path.pop();
        }
    });
}

#[inline]
#[allow(dead_code)]
fn descend_radix(parent: &Trie, board: &RadixBoggleBoard, mut word: &mut Vec<u8>, mut path: &mut Vec<usize>, mut words: &mut BitSet) {
    for (trie, i) in parent.iter() {
        word.push(i);

        for pos in board.neighbors(*path.last().unwrap(), i) {

            if !path.contains(&pos) {
                path.push(pos);

                match trie.node_type() {
                    NodeType::Word(id) => {
                        if !words.get(id) {
                            words.add(id);
                            let mut found: Vec<u8> = word.clone();
                            for b in found.iter_mut() {
                                *b += 'a' as u8;
                            }

                            let s = unsafe { std::str::from_utf8_unchecked(&found) }.replace("q", "qu");
                            println!("{}", s);
                        }
                    },
                    _ => ()
                }


                descend_radix(trie, board, &mut word, &mut path, &mut words);
                path.pop();
            }
        }
        word.pop();
    }
}


//==============================================================================


fn main() {
    use std::time::Instant;

    use std::io::BufReader;
    use std::io::BufRead;
    use std::io::Write;
    use std::fs::File;

    let start = Instant::now();

    let mut trie = Trie::new();
    
    {
        //let f = File::open("wordlists/yawl-0.3.2.03.list");
        let f = File::open("wordlists/enable1.txt");

        /*
         * Boggle rules state words must be at least three characters, also
         * there is no Q face on any die, it's replaced with a Qu. As any
         * word containing Q not followed by u is illegal, we filter them out
         * an replace 'qu' with 'q' here, and repace 'q' with 'qu' when solving.
         * (This seperates the game rules from trie and board represention)
         */
        match f {
            Ok(file) => {

                /* 
                 * This is ugly, it would be better to have another struct to
                 * represent a dictionary and hold the root trie, the id counter,
                 * and any other interesting statistics.
                 */
                let mut id = 0;

                let reader = BufReader::new(&file);
                for word in reader.lines()
                                  .map(|result| result.unwrap())
                                  .filter(|ref line| line.len() >= 3)
                                  .filter(|ref line| {
                                      let mut iter = line.chars();
                                      while let Some(c) = iter.next() {
                                          if c == 'q' || c == 'q' {
                                            if let Some(n) = iter.next() {
                                                if !(n == 'u' || n == 'U') {
                                                    // q not followed by u
                                                    return false;
                                                }
                                            } else {
                                                // line ends in q
                                                return false;
                                            }
                                          }
                                      }
                                      true
                                  })
                                  .map(|ref line| line.replace("qu", "q"))
                {
                    if trie.insert(&word, id) {
                        id += 1;
                    }
                }
            },
            Err(e) => panic!("{:?}", e)
        }
    }   
    let _ = writeln!(&mut std::io::stderr(), "Build Dictionary: {:?}", start.elapsed());

    let start = Instant::now();

    let simple_board: SimpleBoggleBoard;
    let radix_board: RadixBoggleBoard;
    {
        let f = File::open("boards/256x256.txt");
        //let f = File::open("boards/1024x1024.txt");

        match f {
            Ok(mut file) => {
                use std::io::Read;
                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();
                let s: &str = &buf;
                simple_board = SimpleBoggleBoard::read(s.lines()).unwrap();

                radix_board = RadixBoggleBoard::from(&simple_board);
            },
            Err(e) => panic!("{:?}", e)
        }
    }
    let _ = writeln!(&mut std::io::stderr(), "Build Board: {:?}", start.elapsed());

    // Leaving this here as poor man's perf tests

    /*let start = Instant::now();
    solve(&trie, &simple_board);
    let _ = writeln!(&mut std::io::stderr(), "Sequential Solve (Simple): {:?}", start.elapsed());*/

    /*let start = Instant::now();
    par_solve(&trie, &simple_board);
    let _ = writeln!(&mut std::io::stderr(), "Paralell Solve (Simple): {:?}", start.elapsed());*/

    let start = Instant::now();
    solve_radix(&trie, &radix_board);
    let _ = writeln!(&mut std::io::stderr(), "Sequential Solve (Radix): {:?}", start.elapsed());

    /*let start = Instant::now();
    par_solve_radix(&trie, &radix_board);
    let _ = writeln!(&mut std::io::stderr(), "Paralell Solve (Radix): {:?}", start.elapsed());*/
}