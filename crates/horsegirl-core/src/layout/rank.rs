//! The rank phase: network simplex.
//!
//! The method is the one that Gansner, Koutsofios, North and Vo describe in
//! "A Technique for Drawing Directed Graphs" (1993). It finds the ranks with
//! the smallest sum of link lengths, so few links pass a rank between their
//! ends. This code computes each cut value from the definition. That costs
//! the number of nodes times the number of links for each pivot, which is
//! small for a diagram that a person reads.

/// Gives each node a rank so that every link goes to a higher rank.
///
/// `links` has no loop and no link from a node to itself. The lowest rank of
/// each connected part is 0.
pub(super) fn rank(count: usize, links: &[(usize, usize)]) -> Vec<usize> {
    let mut ranks = longest_path(count, links);
    let mut tree = tight_tree(&mut ranks, links);
    // A pivot that changes no rank can repeat, so the pivots have a limit.
    // The ranks are valid after every pivot. The search starts after the last
    // link that left, which makes a repeat rare.
    let mut start = 0;
    for _ in 0..4 * count {
        let Some((leave, tail)) = leaving(count, links, &tree, start) else {
            break;
        };
        let entering = links
            .iter()
            .enumerate()
            .filter(|&(_, &(a, b))| !tail[a] && tail[b])
            .min_by_key(|&(e, _)| slack(&ranks, links[e]));
        let Some((enter, _)) = entering else {
            break;
        };
        // The entering link goes into the tail part. Moving every other node
        // to a higher rank by the slack makes the link tight and keeps every
        // link valid.
        let by = slack(&ranks, links[enter]);
        for (v, rank) in ranks.iter_mut().enumerate() {
            if !tail[v] {
                *rank += by;
            }
        }
        tree[leave] = false;
        tree[enter] = true;
        start = leave + 1;
    }
    settle(&mut ranks, links);
    ranks
}

fn slack(ranks: &[usize], (a, b): (usize, usize)) -> usize {
    ranks[b] - ranks[a] - 1
}

/// The first ranks: each node one rank after the last node that links to it.
fn longest_path(count: usize, links: &[(usize, usize)]) -> Vec<usize> {
    let mut out = vec![Vec::new(); count];
    let mut waiting = vec![0_usize; count];
    for &(a, b) in links {
        out[a].push(b);
        waiting[b] += 1;
    }
    let mut ranks = vec![0; count];
    let mut ready: Vec<usize> = (0..count).filter(|&v| waiting[v] == 0).collect();
    while let Some(v) = ready.pop() {
        for &w in &out[v] {
            ranks[w] = ranks[w].max(ranks[v] + 1);
            waiting[w] -= 1;
            if waiting[w] == 0 {
                ready.push(w);
            }
        }
    }
    ranks
}

/// Marks a spanning tree of tight links for each connected part, and changes
/// the ranks until one exists. A link is tight when its ends are one rank
/// apart.
fn tight_tree(ranks: &mut [usize], links: &[(usize, usize)]) -> Vec<bool> {
    let count = ranks.len();
    let mut tree = vec![false; links.len()];
    let mut inside = vec![false; count];
    for root in 0..count {
        if inside[root] {
            continue;
        }
        inside[root] = true;
        loop {
            let mut grew = true;
            while grew {
                grew = false;
                for (e, &(a, b)) in links.iter().enumerate() {
                    if inside[a] != inside[b] && slack(ranks, (a, b)) == 0 {
                        inside[a] = true;
                        inside[b] = true;
                        tree[e] = true;
                        grew = true;
                    }
                }
            }
            // The closest link out of the tree. The nodes on the side of its
            // start move to higher ranks until it is tight. Nodes of other
            // parts can move with them, which changes nothing between two
            // nodes of one part.
            let closest = links
                .iter()
                .filter(|&&(a, b)| inside[a] != inside[b])
                .min_by_key(|&&link| slack(ranks, link));
            let Some(&(a, b)) = closest else {
                break;
            };
            let by = slack(ranks, (a, b));
            let start_side = inside[a];
            for v in 0..count {
                if inside[v] == start_side {
                    ranks[v] += by;
                }
            }
        }
    }
    tree
}

/// Finds a tree link that the ranks gain from removing, and the nodes on the
/// side of its start.
///
/// Without the link, the tree of its part falls into a tail part and a head
/// part. The link leaves when more links go from the head part to the tail
/// part than the other way: then moving the two parts together shortens more
/// links than it stretches.
fn leaving(
    count: usize,
    links: &[(usize, usize)],
    tree: &[bool],
    start: usize,
) -> Option<(usize, Vec<bool>)> {
    let from_start = (0..links.len()).map(|offset| (start + offset) % links.len());
    for e in from_start.filter(|&e| tree[e]) {
        let tail = tail_part(count, links, tree, e);
        let out = links.iter().filter(|&&(a, b)| tail[a] && !tail[b]).count();
        let back = links.iter().filter(|&&(a, b)| !tail[a] && tail[b]).count();
        if back > out {
            return Some((e, tail));
        }
    }
    None
}

/// The nodes that the tree joins to the start of `cut` without `cut`.
fn tail_part(count: usize, links: &[(usize, usize)], tree: &[bool], cut: usize) -> Vec<bool> {
    let mut near = vec![Vec::new(); count];
    for (e, &(a, b)) in links.iter().enumerate() {
        if tree[e] && e != cut {
            near[a].push(b);
            near[b].push(a);
        }
    }
    flood(&near, links[cut].0)
}

fn flood(near: &[Vec<usize>], from: usize) -> Vec<bool> {
    let mut reached = vec![false; near.len()];
    reached[from] = true;
    let mut stack = vec![from];
    while let Some(v) = stack.pop() {
        for &w in &near[v] {
            if !reached[w] {
                reached[w] = true;
                stack.push(w);
            }
        }
    }
    reached
}

/// Moves each connected part so that its lowest rank is 0.
fn settle(ranks: &mut [usize], links: &[(usize, usize)]) {
    let mut near = vec![Vec::new(); ranks.len()];
    for &(a, b) in links {
        near[a].push(b);
        near[b].push(a);
    }
    let mut done = vec![false; ranks.len()];
    for root in 0..ranks.len() {
        if done[root] {
            continue;
        }
        let part = flood(&near, root);
        let members = || (0..ranks.len()).filter(|&v| part[v]);
        let lowest = members().map(|v| ranks[v]).min().unwrap_or(0);
        for v in members() {
            ranks[v] -= lowest;
            done[v] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rank;

    fn length(ranks: &[usize], links: &[(usize, usize)]) -> usize {
        links.iter().map(|&(a, b)| ranks[b] - ranks[a]).sum()
    }

    #[test]
    fn a_chain_has_one_rank_for_each_node() {
        assert_eq!(rank(3, &[(0, 1), (1, 2)]), [0, 1, 2]);
    }

    #[test]
    fn no_link_gives_rank_zero() {
        assert_eq!(rank(3, &[]), [0, 0, 0]);
        assert_eq!(rank(0, &[]), [0; 0]);
    }

    // The longest path puts node 0 in rank 0, two ranks from node 3.
    #[test]
    fn a_source_moves_next_to_what_it_links_to() {
        assert_eq!(rank(4, &[(0, 3), (1, 2), (2, 3)]), [1, 0, 1, 2]);
    }

    #[test]
    fn each_connected_part_starts_at_rank_zero() {
        assert_eq!(rank(5, &[(0, 1), (1, 2), (3, 4)]), [0, 1, 2, 0, 1]);
    }

    /// The smallest sum of link lengths over every choice of ranks.
    fn smallest(count: usize, links: &[(usize, usize)]) -> usize {
        let mut ranks = vec![0; count];
        let mut best = usize::MAX;
        loop {
            if links.iter().all(|&(a, b)| ranks[a] < ranks[b]) {
                best = best.min(length(&ranks, links));
            }
            let Some(digit) = ranks.iter().position(|&r| r + 1 < count) else {
                return best;
            };
            ranks[..digit].fill(0);
            ranks[digit] += 1;
        }
    }

    // 200 graphs of up to six nodes, from a fixed sequence of numbers. A link
    // goes from a lower index to a higher one, so no graph has a loop.
    #[test]
    fn the_sum_of_the_link_lengths_is_the_smallest() {
        let mut state: u32 = 1;
        let mut next = |below: usize| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 16) as usize % below
        };
        for _ in 0..200 {
            let count = 2 + next(5);
            let links: Vec<(usize, usize)> = (0..=next(9))
                .map(|_| {
                    let a = next(count - 1);
                    (a, a + 1 + next(count - 1 - a))
                })
                .collect();
            let ranks = rank(count, &links);
            assert!(links.iter().all(|&(a, b)| ranks[a] < ranks[b]), "{links:?}");
            assert_eq!(length(&ranks, &links), smallest(count, &links), "{links:?}");
        }
    }

    #[test]
    fn a_link_stated_twice_stays_valid() {
        assert_eq!(rank(2, &[(0, 1), (0, 1)]), [0, 1]);
    }
}
