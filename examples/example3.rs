use lib_wfa2::affine_wavefront::{Distance, MemoryMode};

pub fn main() {
    println!("Example3 - UltraLow memory mode\n");
    // Create edit distance aligner with no heuristic, using UltraLow memory mode
    let aligner = Distance::Edit.create_aligner(None, Some(&MemoryMode::Ultralow));

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
    println!("Memory mode: {:?}", aligner.get_memory_mode());
}
