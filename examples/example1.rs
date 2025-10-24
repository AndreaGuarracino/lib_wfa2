use lib_wfa2::affine_wavefront::AffineWavefronts;

pub fn main() {
    println!("Example1\n");
    // Create edit distance aligner with no heuristic
    let aligner = AffineWavefronts::new_aligner_edit(None);

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
