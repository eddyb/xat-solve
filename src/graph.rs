use std::convert::TryInto;
use std::mem;
use std::num::NonZeroU16;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Node(NonZeroU16);

#[derive(Clone)]
enum NodeData {
    Known(bool),
    Unknown,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Contradiction;

#[derive(Clone)]
pub struct Graph {
    nodes: Vec<NodeData>,
    one_of_groups: Vec<(Vec<Node>, bool)>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            nodes: vec![NodeData::Known(false)],
            one_of_groups: vec![],
        }
    }
}

impl Graph {
    pub fn new_node(&mut self) -> Node {
        let node = Node(NonZeroU16::new(self.nodes.len().try_into().unwrap()).unwrap());
        self.nodes.push(NodeData::Unknown);
        node
    }

    pub fn get_node(&self, node: Node) -> Option<bool> {
        match self.nodes[node.0.get() as usize] {
            NodeData::Known(value) => Some(value),
            NodeData::Unknown => None,
        }
    }

    pub fn set_node(&mut self, node: Node, value: bool) -> Result<(), Contradiction> {
        let old = mem::replace(
            &mut self.nodes[node.0.get() as usize],
            NodeData::Known(value),
        );
        match old {
            NodeData::Known(old_value) => {
                if old_value != value {
                    return Err(Contradiction);
                }
            }
            NodeData::Unknown => {
                // FIXME(eddyb) find a way to signal the one-of groups
                // that this nodes is a part of (is that even worth it?).
            }
        }
        Ok(())
    }

    pub fn require_at_most_one_of(&mut self, nodes: impl Iterator<Item = Node>) {
        self.one_of_groups.push((nodes.collect(), false));
    }

    pub fn require_exactly_one_of(&mut self, nodes: impl Iterator<Item = Node>) {
        self.one_of_groups.push((nodes.collect(), true));
    }

    pub fn solve(mut self) -> Result<Self, Contradiction> {
        let mut one_of_groups = mem::replace(&mut self.one_of_groups, vec![]);

        loop {
            let mut changed = false;

            // Clean up solved one-of groups.
            one_of_groups.retain(|(nodes, _)| !nodes.is_empty());

            for (nodes, exact) in &mut one_of_groups {
                let mut found_true = Ok(false);

                // Keep only the unknown nodes.
                nodes.retain(|&node| {
                    let known = node.get(&mut self);

                    if known == Some(true) {
                        if found_true == Ok(true) {
                            found_true = Err(Contradiction);
                        } else {
                            found_true = Ok(true);
                        }
                    }

                    changed |= known.is_some();

                    known.is_none()
                });

                // If one node in the one-of group is `true`, all others must be `false`.
                if found_true? {
                    for node in nodes.drain(..) {
                        node.set(&mut self, false)?;
                    }
                    continue;
                }

                if *exact {
                    if nodes.is_empty() {
                        return Err(Contradiction);
                    }

                    // Only one unknown node left, it must be `true`.
                    if let [node] = nodes[..] {
                        node.set(&mut self, true)?;
                        nodes.clear();
                        changed = true;
                    }
                }
            }

            if changed {
                continue;
            }

            // Take advantage of some one-of groups being included in larger
            // ones, to make some progress.
            one_of_groups.sort_by_key(|(nodes, _)| nodes.len());
            for i in 0..one_of_groups.len() {
                let (needles, haystacks) = one_of_groups[i..].split_first_mut().unwrap();
                let (needles, exact) = needles;
                if !*exact {
                    continue;
                }
                for (haystack, _) in haystacks {
                    if haystack.len() <= needles.len() {
                        continue;
                    }
                    if needles.iter().all(|node| haystack.contains(&node)) {
                        // Everything outside of the smaller one-of group must be `false`.
                        for node in haystack.drain(..) {
                            if !needles.contains(&node) {
                                node.set(&mut self, false)?;
                            }
                        }
                        changed = true;
                        break;
                    }
                }
                if changed {
                    break;
                }
            }

            if changed {
                continue;
            }

            if one_of_groups.is_empty() {
                break;
            }

            // As a desperate measure, brute-force.
            for (nodes, exact) in &one_of_groups {
                if !*exact {
                    continue;
                }
                let candidate = nodes[0];

                let mut alternate = self.clone();
                alternate.one_of_groups = one_of_groups.clone();
                candidate.set(&mut alternate, true).unwrap();
                match alternate.solve() {
                    Ok(alternate) => return Ok(alternate),
                    Err(Contradiction) => {
                        candidate.set(&mut self, false).unwrap();
                        break;
                    }
                }
            }
        }

        self.one_of_groups = one_of_groups;

        Ok(self)
    }
}

/// Convenience methods that call the respective `Graph` methods.
impl Node {
    pub fn new(g: &mut Graph) -> Self {
        g.new_node()
    }

    pub fn get(self, g: &Graph) -> Option<bool> {
        g.get_node(self)
    }

    pub fn set(self, g: &mut Graph, value: bool) -> Result<(), Contradiction> {
        g.set_node(self, value)
    }
}
