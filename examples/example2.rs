use lib_wfa2::affine_wavefront::{Distance, HeuristicStrategy};

pub fn main() {
    println!("Example2\n");

    // Create edit distance aligner with banded static heuristic
    let aligner = Distance::Edit.create_aligner(Some(&HeuristicStrategy::BandedStatic {
        band_min_k: -15,
        band_max_k: 15,
    }));

    // pattern means query
    let pattern = b"TCTTTACTCGCGCGTTGGAGAAATACAATAGT";

    // Text means reference
    let text = b"TCTATACTGCGCGTTTGGAGAAATAAAATAGT";

    let status = aligner.align(pattern, text);

    println!("Pattern: {}", String::from_utf8_lossy(pattern));
    println!("Text:    {}\n", String::from_utf8_lossy(text));

    println!("Status: {:?}", status);
    println!("Score: {}", aligner.score());
    println!("Cigar: {}", String::from_utf8_lossy(aligner.cigar()));
}
