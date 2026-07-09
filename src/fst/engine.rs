//! Low-level weighted-FST operations used by the parity engine.
//!
//! These wrap `rustfst` to reproduce the two behaviors NeMo's `normalize.py`
//! relies on but which `rustfst` does not match bit-for-bit out of the box:
//!
//! 1. **`rm_epsilon` before shortest-path.** Composition introduces epsilon
//!    transitions that otherwise break tropical weight accumulation, silently
//!    dropping the lowest-cost path NeMo would pick.
//! 2. **A custom tropical DAG shortest-path.** `rustfst::shortest_path`
//!    diverges from OpenFST on complex compositions. Because a byte-acceptor
//!    composed with an acyclic grammar is itself acyclic, a topological
//!    relaxation reproduces OpenFST's result exactly.
//!
//! See `docs/NEMO_PARITY.md` for how these were isolated.

use rustfst::algorithms::compose::compose;
use rustfst::algorithms::rm_epsilon::rm_epsilon;
use rustfst::prelude::*;

/// Build a linear FST that accepts exactly the bytes of `s` (input == output).
///
/// Working at the byte level keeps the engine encoding-agnostic: Chinese,
/// Devanagari, and ASCII all flow through the same UTF-8 byte labels.
fn byte_acceptor(s: &str) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::new();
    let mut prev = fst.add_state();
    fst.set_start(prev).expect("start state");
    for &b in s.as_bytes() {
        let next = fst.add_state();
        fst.add_tr(
            prev,
            Tr::new(b as u32, b as u32, TropicalWeight::one(), next),
        )
        .expect("add transition");
        prev = next;
    }
    fst.set_final(prev, TropicalWeight::one())
        .expect("final state");
    fst
}

/// Return the output-label string of the lowest-weight path through an acyclic
/// FST, or `None` if it has no accepting path.
///
/// Iterative-DFS topological order, then a single relaxation pass
/// (`dist[next] = min(dist[next], dist[s] + w)`) followed by a backtrack over
/// the recorded predecessors. Matches OpenFST's tropical shortest-path on the
/// acyclic FSTs this engine produces.
fn shortest_output(fst: &VectorFst<TropicalWeight>) -> Option<String> {
    let start = fst.start()?;
    let n = fst.num_states();

    // Topological order via iterative DFS (post-order, then reversed).
    let mut order = Vec::new();
    let mut visited = vec![0u8; n];
    let mut stack = vec![(start as usize, false)];
    while let Some((s, done)) = stack.pop() {
        if done {
            order.push(s);
            continue;
        }
        if visited[s] != 0 {
            continue;
        }
        visited[s] = 1;
        stack.push((s, true));
        for tr in fst.get_trs(s as StateId).unwrap().iter() {
            let ns = tr.nextstate as usize;
            if visited[ns] == 0 {
                stack.push((ns, false));
            }
        }
    }
    order.reverse();

    // Relax edges in topological order.
    let inf = f32::INFINITY;
    let mut dist = vec![inf; n];
    let mut pred: Vec<Option<(usize, u32)>> = vec![None; n];
    dist[start as usize] = 0.0;
    for &s in &order {
        if dist[s] == inf {
            continue;
        }
        for tr in fst.get_trs(s as StateId).unwrap().iter() {
            let w = dist[s] + tr.weight.value();
            let ns = tr.nextstate as usize;
            if w < dist[ns] {
                dist[ns] = w;
                pred[ns] = Some((s, tr.olabel));
            }
        }
    }

    // Pick the cheapest final state (edge weight + final weight).
    let mut best_final = None;
    let mut best_weight = inf;
    for (f, &df) in dist.iter().enumerate() {
        if let Ok(Some(fw)) = fst.final_weight(f as StateId) {
            let total = df + fw.value();
            if total < best_weight {
                best_weight = total;
                best_final = Some(f);
            }
        }
    }

    // Backtrack, collecting non-epsilon output labels.
    let mut f = best_final?;
    let mut labels = Vec::new();
    while let Some((p, olabel)) = pred[f] {
        if olabel != 0 {
            labels.push(olabel as u8);
        }
        f = p;
    }
    labels.reverse();
    Some(String::from_utf8_lossy(&labels).to_string())
}

/// Apply a transducer to `input`: compose, remove epsilons, take the tropical
/// shortest path's output. Returns `None` if the input is not in the domain
/// (empty composition) or the shortest path emits nothing.
pub fn apply(fst: &VectorFst<TropicalWeight>, input: &str) -> Option<String> {
    let mut composed: VectorFst<TropicalWeight> =
        compose(byte_acceptor(input), fst.clone()).ok()?;
    if composed.num_states() == 0 || composed.start().is_none() {
        return None;
    }
    rm_epsilon(&mut composed).ok()?;
    let out = shortest_output(&composed)?;
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}
