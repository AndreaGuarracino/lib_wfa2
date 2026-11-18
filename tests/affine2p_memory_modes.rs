// Gap-affine2p in every memory mode: same score, CIGAR spans both sequences.
// High uses the Singletrack backtrace (WFA2-lib #150); a binding layout mismatch crashes here.
use lib_wfa2::affine_wavefront::{AffineWavefronts, MemoryMode};

fn seqs() -> (Vec<u8>, Vec<u8>) {
    let mut x: u64 = 42;
    let mut next = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
    let a: Vec<u8> = (0..5000).map(|_| b"ACGT"[(next() % 4) as usize]).collect();
    let mut b = Vec::new();
    for &c in &a {
        match next() % 100 {
            0..=2 => b.push(b"ACGT"[(next() % 4) as usize]), // mismatch
            3 => {}                                            // deletion
            4 => { b.push(c); b.extend((0..(next() % 30)).map(|_| b'A')) } // insertion
            _ => b.push(c),
        }
    }
    (a, b)
}

// (pattern a, text b) consumed; WFA2 'I' consumes the text, 'D' the pattern
fn spans(cigar: &[u8]) -> (usize, usize) {
    cigar.iter().fold((0, 0), |(q, t), op| match op {
        b'M' | b'X' | b'=' => (q + 1, t + 1),
        b'I' => (q, t + 1),
        b'D' => (q + 1, t),
        _ => panic!("unexpected CIGAR op {}", *op as char),
    })
}

#[test]
fn affine2p_scores_agree_across_memory_modes() {
    let (a, b) = seqs();
    let mut scores = Vec::new();
    for (name, mode) in [("high", MemoryMode::High), ("medium", MemoryMode::Medium), ("low", MemoryMode::Low), ("ultralow", MemoryMode::Ultralow)] {
        let al = AffineWavefronts::with_penalties_affine2p_and_memory_mode(0, 5, 8, 2, 24, 1, mode);
        al.align(&a, &b);
        let (q, t) = spans(al.cigar());
        assert_eq!((q, t), (a.len(), b.len()), "CIGAR span in {name}");
        scores.push(al.score());
    }
    assert!(scores.windows(2).all(|w| w[0] == w[1]), "scores differ across modes: {scores:?}");
}
