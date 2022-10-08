//! An implementation of D* Lite: second version (optimized version) as
//! described in <https://www.cs.cmu.edu/~maxim/files/dlite_tro05.pdf>
//!
//! Future optimization attempt ideas:
//! - Use a different priority queue (e.g. fibonacci heap)
//! - Use FxHash instead of the default hasher
//! - Have a `cost(a: Vertex, b: Vertex)` function instead of having the cost be stored in `Edge`

use priority_queue::PriorityQueue;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::{
    borrow::Cow,
    cmp,
    collections::HashMap,
    hash::Hash,
    ops::{Add, Deref},
};

#[derive(Debug)]
pub struct VertexScore<W: Default + num_traits::Bounded + Debug> {
    pub g: W,
    pub rhs: W,
}

impl<W: Default + num_traits::Bounded + Debug> Default for VertexScore<W> {
    fn default() -> Self {
        Self {
            g: W::max_value(),
            rhs: W::max_value(),
        }
    }
}

/// The D* Lite pathfinding algorithm
pub struct DStarLite<
    'a,
    N: Eq + Hash + Clone,
    W: PartialOrd + Eq + Default + Copy + num_traits::Bounded + Debug,
    HeuristicFn: Fn(&N, &N) -> W,
    SuccessorsFn: Fn(&N) -> Vec<EdgeTo<N, W>>,
    PredcessorsFn: Fn(&N) -> Vec<EdgeTo<N, W>>,
> {
    /// Rough estimate of how close we are to the goal. Lower = closer.
    pub heuristic: HeuristicFn,
    /// Get the nodes that can be reached from the current one
    pub successors: SuccessorsFn,
    /// Get the nodes that would direct us to the current node
    pub predecessors: PredcessorsFn,

    pub start: Cow<'a, N>,
    start_last: Cow<'a, N>,

    goal: N,

    queue: PriorityQueue<N, Priority<W>>,
    k_m: W,
    vertex_scores: HashMap<N, VertexScore<W>>,
    /// This is just here so we can reference it. It should never be modified.
    default_score: VertexScore<W>,

    /// A list of edges and costs that we'll be updating next time.
    pub updated_edge_costs: Vec<(Edge<'a, N, W>, W)>,
}

pub struct Edge<'a, N: Eq + Hash + Clone, W: PartialOrd + Copy> {
    pub predecessor: Cow<'a, N>,
    pub successor: Cow<'a, N>,
    pub cost: W,
}

pub struct EdgeTo<N: Eq + Hash + Clone, W: PartialOrd + Copy> {
    pub target: N,
    pub cost: W,
}

// rust does lexicographic ordering by default when we derive Ord
#[derive(Eq, PartialEq, Debug)]
pub struct Priority<W>(W, W)
where
    W: PartialOrd + Debug;

impl<W: PartialOrd + Debug> PartialOrd for Priority<W> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 < other.0 {
            Some(std::cmp::Ordering::Less)
        } else if self.0 > other.0 {
            Some(std::cmp::Ordering::Greater)
        } else if self.1 < other.1 {
            Some(std::cmp::Ordering::Less)
        } else if self.1 > other.1 {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}
impl<W: PartialOrd + Debug + Eq> Ord for Priority<W> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("Partial compare should not fail for Priority")
    }
}

#[derive(Debug)]
pub struct NoPathError;
impl Error for NoPathError {}
impl Display for NoPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No path found")
    }
}

impl<
        'a,
        N: Eq + Hash + Clone + Debug,
        W: PartialOrd + Eq + Add<Output = W> + Default + Copy + num_traits::bounds::Bounded + Debug,
        HeuristicFn: Fn(&N, &N) -> W,
        SuccessorsFn: Fn(&N) -> Vec<EdgeTo<N, W>>,
        PredecessorsFn: Fn(&N) -> Vec<EdgeTo<N, W>>,
    > DStarLite<'a, N, W, HeuristicFn, SuccessorsFn, PredecessorsFn>
{
    fn score(&self, node: &N) -> &VertexScore<W> {
        self.vertex_scores.get(node).unwrap_or(&self.default_score)
    }
    fn score_mut(&mut self, node: &N) -> &mut VertexScore<W> {
        self.vertex_scores.entry(node.clone()).or_default()
    }

    fn calculate_key(&self, s: &N) -> Priority<W> {
        let s_score = self.score(s);
        let min_score = if s_score.g < s_score.rhs {
            s_score.g
        } else {
            s_score.rhs
        };
        Priority(
            if min_score == W::max_value() {
                min_score
            } else {
                min_score + (self.heuristic)(&self.start, s) + self.k_m
            },
            min_score,
        )
    }

    pub fn new(
        start: N,
        goal: N,
        heuristic: HeuristicFn,
        successors: SuccessorsFn,
        predecessors: PredecessorsFn,
    ) -> Self {
        let mut queue = PriorityQueue::with_capacity(1);
        // Vertex<N, W>, Priority<W>

        let mut vertex_scores = HashMap::new();
        vertex_scores.insert(
            goal.clone(),
            VertexScore {
                g: W::max_value(),
                rhs: W::default(),
            },
        );
        queue.push(
            goal.clone(),
            Priority(heuristic(&start, &goal), W::default()),
        );

        let mut s = Self {
            start: Cow::Owned(start.clone()),
            start_last: Cow::Owned(start),

            goal,

            heuristic,
            successors,
            predecessors,
            default_score: VertexScore::default(),

            queue,
            k_m: W::default(),
            vertex_scores,

            updated_edge_costs: Vec::new(),
        };
        s.compute_shortest_path();
        s
    }

    pub fn update_vertex(&mut self, u: &N) {
        let VertexScore { g, rhs } = self.score(u);
        // if(g(u)) != rhs(u) AND u is in U) U.Update(u, calculate_key(u))
        if g != rhs && self.queue.get(u).is_some() {
            self.queue.change_priority(u, self.calculate_key(u));
        } else if g != rhs && self.queue.get(u).is_none() {
            self.queue.push(u.clone(), self.calculate_key(u));
        } else if g == rhs && self.queue.get(u).is_some() {
            self.queue.remove(u);
        }
    }

    fn compute_shortest_path(&mut self) {
        while {
            let score = self.score(&self.start);
            if let Some(queue_top) = self.queue.peek() {
                (queue_top.1 < &self.calculate_key(&self.start)) || (score.rhs > score.g)
            } else {
                false
            }
        } {
            let (u, k_old) = self.queue.pop().unwrap();
            let k_new = self.calculate_key(&u);
            if k_old < k_new {
                self.queue.change_priority(&u, k_new);
                continue;
            }
            let u_score = self.score_mut(&u);
            if u_score.g > u_score.rhs {
                u_score.g = u_score.rhs;
                let g_u = u_score.g;
                self.queue.remove(&u);
                for s in (self.predecessors)(&u) {
                    let target_score = self.score_mut(&s.target);
                    if s.cost + g_u < target_score.rhs {
                        target_score.rhs = s.cost + g_u;
                    }
                    // TODO: i think this can be moved up, but i'm not 100% sure it won't break anything
                    self.update_vertex(&s.target);
                }
            } else {
                let g_old = u_score.g;
                u_score.g = W::max_value();
                // for all s in Pred(u) + {u}
                //   if (rhs(s) = c(s, u) + g_old)
                //     if (s != s_goal) rhs(s) = min s' in Succ(s) (c(s, s') + g(s'))
                //   update_vertex(s)
                for s in ((self.predecessors)(&u)).into_iter().chain(
                    [EdgeTo {
                        target: u,
                        cost: W::default(),
                    }]
                    .into_iter(),
                ) {
                    if self.score(&s.target).rhs == s.cost + g_old && s.target != self.goal {
                        let mut lowest_score = W::max_value();
                        for s_prime in (self.successors)(&s.target) {
                            let s_prime_score = s_prime.cost + self.score(&s_prime.target).g;
                            if s_prime_score < lowest_score {
                                lowest_score = s_prime_score;
                            }
                        }
                        self.score_mut(&s.target).rhs = lowest_score;
                    }
                    self.update_vertex(&s.target);
                }
            }
        }
    }

    pub fn update_from_updated_edges(&mut self) {
        self.k_m = self.k_m + (self.heuristic)(&self.start, &self.start_last);
        self.start_last = self.start.clone();

        while let Some((mut edge, new_cost)) = self.updated_edge_costs.pop() {
            let old_cost = edge.cost;
            edge.cost = new_cost;
            let target_score = self.score_mut(&edge.successor);
            if old_cost > new_cost {
                if edge.cost + target_score.g < target_score.rhs {
                    target_score.rhs = edge.cost + target_score.g;
                }
            } else if target_score.rhs == old_cost + target_score.g {
                let g_score = target_score.g;
                if edge.successor.deref() != &self.goal {
                    let successors = (self.successors)(&edge.successor);
                    let mut lowest_score = W::max_value();
                    for s in successors {
                        let score = s.cost + g_score;
                        if score < lowest_score {
                            lowest_score = score;
                        }
                    }
                    self.score_mut(&edge.successor).rhs = lowest_score;
                }
            }
            self.update_vertex(&edge.successor);
        }
    }

    /// Return the next vertex to visit and set our current position to be there.
    pub fn try_next(&mut self) -> Result<Option<&N>, NoPathError> {
        if self.start.deref() == &self.goal {
            return Ok(None);
        }

        let start_score = self.score(&self.start);
        if start_score.rhs == W::max_value() {
            return Err(NoPathError);
        }

        let get_score = |edge: &EdgeTo<N, W>| -> W {
            let g_score = self.score(&edge.target).g;
            if g_score == W::max_value() {
                W::max_value()
            } else {
                edge.cost + g_score
            }
        };

        *self.start.to_mut() = (self.successors)(&self.start)
            .into_iter()
            .min_by(|a, b| get_score(a).partial_cmp(&get_score(b)).unwrap())
            .expect("No possible successors")
            .target;
        return Ok(Some(self.start.as_ref()));
    }

    // /// Change our current position.
    // pub fn set_start(&mut self, s: Vertex<N, W>) {
    //     *self.start.to_mut() = s;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dstarlite() {
        let maze = [
            [0, 1, 0, 0, 0],
            [0, 1, 0, 1, 0],
            [0, 0, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 0, 1, 0, 0],
        ];
        let width = maze[0].len();
        let height = maze.len();

        fn heuristic(a: &(usize, usize), b: &(usize, usize)) -> usize {
            ((a.0 as isize - b.0 as isize).abs() + (a.1 as isize - b.1 as isize).abs()) as usize
        }
        let successors = |a: &(usize, usize)| -> Vec<EdgeTo<(usize, usize), usize>> {
            let mut successors = Vec::with_capacity(4);
            let (x, y) = *a;

            if x > 0 && maze[y][x - 1] == 0 {
                successors.push(EdgeTo {
                    target: ((x - 1, y)),
                    cost: 1,
                });
            }
            if x < width - 1 && maze[y][x + 1] == 0 {
                successors.push(EdgeTo {
                    target: ((x + 1, y)),
                    cost: 1,
                });
            }
            if y > 0 && maze[y - 1][x] == 0 {
                successors.push(EdgeTo {
                    target: ((x, y - 1)),
                    cost: 1,
                });
            }
            if y < height - 1 && maze[y + 1][x] == 0 {
                successors.push(EdgeTo {
                    target: ((x, y + 1)),
                    cost: 1,
                });
            }

            successors
        };
        let predecessors = |a: &(usize, usize)| -> Vec<EdgeTo<(usize, usize), usize>> {
            let mut predecessors = Vec::with_capacity(4);
            let (x, y) = *a;

            if x > 0 && maze[y][x - 1] == 0 {
                predecessors.push(EdgeTo {
                    target: ((x - 1, y)),
                    cost: 1,
                });
            }
            if x < width - 1 && maze[y][x + 1] == 0 {
                predecessors.push(EdgeTo {
                    target: ((x + 1, y)),
                    cost: 1,
                });
            }
            if y > 0 && maze[y - 1][x] == 0 {
                predecessors.push(EdgeTo {
                    target: ((x, y - 1)),
                    cost: 1,
                });
            }
            if y < height - 1 && maze[y + 1][x] == 0 {
                predecessors.push(EdgeTo {
                    target: ((x, y + 1)),
                    cost: 1,
                });
            }

            predecessors
        };

        let mut dstar = DStarLite::new((0, 0), (4, 4), heuristic, successors, predecessors);
        assert!(dstar.try_next().unwrap() == Some(&(0, 1)));
        assert!(dstar.try_next().unwrap() == Some(&(0, 2)));
        assert!(dstar.try_next().unwrap() == Some(&(1, 2)));
        assert!(dstar.try_next().unwrap() == Some(&(2, 2)));
        assert!(dstar.try_next().unwrap() == Some(&(2, 1)));
        assert!(dstar.try_next().unwrap() == Some(&(2, 0)));
        assert!(dstar.try_next().unwrap() == Some(&(3, 0)));
        assert!(dstar.try_next().unwrap() == Some(&(4, 0)));
        assert!(dstar.try_next().unwrap() == Some(&(4, 1)));
        assert!(dstar.try_next().unwrap() == Some(&(4, 2)));
        assert!(dstar.try_next().unwrap() == Some(&(4, 3)));
        assert!(dstar.try_next().unwrap() == Some(&(4, 4)));
        assert!(dstar.try_next().unwrap() == None);
    }
}
