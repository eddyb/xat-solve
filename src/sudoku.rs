use crate::graph::{self, Graph};

use std::iter;

fn mk9<T>(mut f: impl FnMut() -> T) -> [T; 9] {
    [f(), f(), f(), f(), f(), f(), f(), f(), f()]
}

#[derive(Copy, Clone, Default)]
pub struct Rules {
    pub anti_knight: bool,
    pub anti_ortho_consecutive: bool,
}

struct Cell([graph::Node; 9]);

impl Cell {
    fn new(g: &mut Graph) -> Self {
        Cell(mk9(|| graph::Node::new(g)))
    }

    fn get(&self, g: &Graph) -> char {
        for (i, node) in self.0.iter().enumerate() {
            if node.get(g) == Some(true) {
                for (j, other) in self.0.iter().enumerate() {
                    assert_eq!(other.get(g), Some(i == j));
                }
                return (b'1' + i as u8) as char;
            }
        }
        '.'
    }

    fn set(&self, g: &mut Graph, c: char) {
        match c {
            '1'..='9' => {
                let value = ((c as u8) - b'1') as usize;
                for (i, node) in self.0.iter().enumerate() {
                    node.set(g, i == value).unwrap();
                }
            }
            _ => {}
        }
    }

    fn require_distinct(g: &mut Graph, a: &Self, b: &Self) {
        for value in 0..9 {
            g.require_at_most_one_of([a, b].iter().map(|cell| cell.0[value]));
        }
    }

    fn require_not_consecutive(g: &mut Graph, a: &Self, b: &Self) {
        for a_value in 0..9 {
            for &b_value in [a_value - 1, a_value + 1].iter() {
                if (0..9).contains(&b_value) {
                    g.require_at_most_one_of(
                        [a.0[a_value as usize], b.0[b_value as usize]]
                            .iter()
                            .copied(),
                    );
                }
            }
        }
    }

    fn require_one_of_each_digit<'a>(g: &mut Graph, cells: impl Iterator<Item = &'a Self> + Clone) {
        assert_eq!(cells.clone().count(), 9);
        for value in 0..9 {
            g.require_exactly_one_of(cells.clone().map(|cell| cell.0[value]));
        }
    }

    fn enforce_rules(&self, g: &mut Graph, _rules: &Rules) {
        // Each `Cell` only has one value.
        g.require_exactly_one_of(self.0.iter().copied())
    }
}

struct Grid([[Cell; 9]; 9]);

impl Grid {
    fn new(g: &mut Graph) -> Self {
        Grid(mk9(|| mk9(|| Cell::new(g))))
    }

    fn get(&self, g: &Graph) -> String {
        self.0.iter().flatten().map(|cell| cell.get(g)).collect()
    }

    fn set(&self, g: &mut Graph, s: &str) {
        for (cell, c) in self.0.iter().flatten().zip(s.chars()) {
            cell.set(g, c);
        }
    }

    fn enforce_rules(&self, g: &mut Graph, rules: &Rules) {
        for cell in self.0.iter().flatten() {
            cell.enforce_rules(g, rules);
        }

        // `Cell`s in the same block must be unique.
        for y in (0..9).step_by(3) {
            for x in (0..9).step_by(3) {
                Cell::require_one_of_each_digit(
                    g,
                    self.0[y..][..3].iter().flat_map(move |row| &row[x..][..3]),
                );
            }
        }

        // `Cell`s on the same row must be unique.
        for y in 0..9 {
            Cell::require_one_of_each_digit(g, self.0[y].iter());
        }

        // `Cell`s on the same column must be unique.
        for x in 0..9 {
            Cell::require_one_of_each_digit(g, self.0.iter().map(move |row| &row[x]));
        }

        if rules.anti_ortho_consecutive {
            for y in 0..9 {
                for x in 0..9 {
                    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)].iter() {
                        let (cx, cy) = (x as i8 + dx, y as i8 + dy);
                        if (0..9).contains(&cx) && (0..9).contains(&cy) {
                            Cell::require_not_consecutive(
                                g,
                                &self.0[y][x],
                                &self.0[cy as usize][cx as usize],
                            );
                        }
                    }
                }
            }
        }

        if rules.anti_knight {
            let two = |a, b| iter::once(a).chain(iter::once(b));
            let every_dir = |dx: i8, dy: i8| {
                two(-dx, dx).flat_map(move |dx| two(-dy, dy).map(move |dy| (dx, dy)))
            };

            for y in 0..9 {
                for x in 0..9 {
                    for (dx, dy) in every_dir(1, 2).chain(every_dir(2, 1)) {
                        let (kx, ky) = (x as i8 + dx, y as i8 + dy);
                        if (0..9).contains(&kx) && (0..9).contains(&ky) {
                            Cell::require_distinct(
                                g,
                                &self.0[y][x],
                                &self.0[ky as usize][kx as usize],
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn solve(s: &str, rules: &Rules) -> String {
    let mut g = Graph::default();
    let grid = Grid::new(&mut g);
    grid.set(&mut g, s);
    grid.enforce_rules(&mut g, rules);
    grid.get(&g.solve().unwrap())
}

#[cfg(test)]
macro_rules! test_solve {
    ($($rule:ident,)* $input:literal, $expected:literal) => {{
        let rules = crate::sudoku::Rules {
            $($rule: true,)*
            ..Default::default()
        };
        assert_eq!(crate::sudoku::solve($input, &rules), $expected)
    }};
}

/// Test cases from http://sudopedia.enjoysudoku.com/Valid_Test_Cases.html.
#[cfg(test)]
mod sudopedia_enjoysudoku_tests {
    #[test]
    fn completed() {
        test_solve!(
            "974236158638591742125487936316754289742918563589362417867125394253649871491873625",
            "974236158638591742125487936316754289742918563589362417867125394253649871491873625"
        );
    }

    #[test]
    fn last_empty_square() {
        test_solve!(
            "2564891733746159829817234565932748617128.6549468591327635147298127958634849362715",
            "256489173374615982981723456593274861712836549468591327635147298127958634849362715"
        );
    }

    #[test]
    fn naked_singles() {
        test_solve!(
            "3.542.81.4879.15.6.29.5637485.793.416132.8957.74.6528.2413.9.655.867.192.965124.8",
            "365427819487931526129856374852793641613248957974165283241389765538674192796512438"
        );
    }

    #[test]
    fn hidden_singles() {
        test_solve!(
            "..2.3...8.....8....31.2.....6..5.27..1.....5.2.4.6..31....8.6.5.......13..531.4..",
            "672435198549178362831629547368951274917243856254867931193784625486592713725316489"
        );
    }
}

/// Test cases from https://www.youtube.com/channel/UCC-UOdK8-mIjxBQm_ot1T-Q.
#[cfg(test)]
#[allow(non_snake_case)]
mod cracking_the_cryptic_tests {
    #[test]
    fn bGSk4rUwhjM() {
        test_solve!(
            "..23....4.7..1..9.1....65..6....98...2..5..7...56....9..85....2.5..6..3.7....31..",
            "962385714573412698184796523637249851429158376815637249398571462251864937746923185"
        );
    }

    #[test]
    fn QNzltTzv0fc() {
        test_solve!(
            anti_knight,
            anti_ortho_consecutive,
            ".....................4.7.....6...5.............4...3.....2.5.....................",
            "973518264425963718861427953316842597758396142294751386649275831182639475537184629"
        );
    }
}
