use crate::bindings::*;
use core::slice;

/// Distance metric for alignment
///
/// This type is primarily for internal use. Most users should use the
/// convenience functions like `create_edit_aligner()` instead of constructing
/// this enum directly.
#[derive(Debug, Clone)]
pub enum Distance {
    Edit,
    GapAffine {
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
    },
    GapAffine2p {
        mismatch: i32,
        gap_opening1: i32,
        gap_extension1: i32,
        gap_opening2: i32,
        gap_extension2: i32,
    },
}

impl Distance {
    pub fn create_aligner(&self, heuristic: Option<&HeuristicStrategy>) -> AffineWavefronts {
        match self {
            Distance::Edit => AffineWavefronts::new_aligner_edit(heuristic),
            Distance::GapAffine {
                mismatch,
                gap_opening,
                gap_extension,
            } => AffineWavefronts::new_aligner_gap_affine(
                *mismatch,
                *gap_opening,
                *gap_extension,
                heuristic,
            ),
            Distance::GapAffine2p {
                mismatch,
                gap_opening1,
                gap_extension1,
                gap_opening2,
                gap_extension2,
            } => AffineWavefronts::new_aligner_gap_affine2p(
                *mismatch,
                *gap_opening1,
                *gap_extension1,
                *gap_opening2,
                *gap_extension2,
                heuristic,
            ),
        }
    }

    /// Convert to u8 for binary serialization
    pub fn to_u8(&self) -> u8 {
        match self {
            Distance::Edit => 0,
            Distance::GapAffine { .. } => 1,
            Distance::GapAffine2p { .. } => 2,
        }
    }

    /// Parse from u8 for binary deserialization
    pub fn from_u8(code: u8) -> Result<Self, String> {
        match code {
            0 => Ok(Distance::Edit),
            1 => Ok(Distance::GapAffine {
                mismatch: 0,
                gap_opening: 0,
                gap_extension: 0,
            }),
            2 => Ok(Distance::GapAffine2p {
                mismatch: 0,
                gap_opening1: 0,
                gap_extension1: 0,
                gap_opening2: 0,
                gap_extension2: 0,
            }),
            _ => Err(format!("Invalid distance code: {}", code)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HeuristicStrategy {
    None,
    BandedStatic {
        band_min_k: std::os::raw::c_int,
        band_max_k: std::os::raw::c_int,
    },
    BandedAdaptive {
        band_min_k: std::os::raw::c_int,
        band_max_k: std::os::raw::c_int,
        score_steps: std::os::raw::c_int,
    },
    WFAdaptive {
        min_wavefront_length: std::os::raw::c_int,
        max_distance_threshold: std::os::raw::c_int,
        score_steps: std::os::raw::c_int,
    },
    XDrop {
        xdrop: std::os::raw::c_int,
        score_steps: std::os::raw::c_int,
    },
    ZDrop {
        zdrop: std::os::raw::c_int,
        score_steps: std::os::raw::c_int,
    },
    WFMash {
        min_wavefront_length: std::os::raw::c_int,
        max_distance_threshold: std::os::raw::c_int,
        score_steps: std::os::raw::c_int,
    },
}

#[derive(Debug, Clone)]
pub enum AlignmentScope {
    ComputeScore,
    Alignment,
    Undefined,
}

impl AlignmentScope {
    pub fn from_scope(val: wfa::alignment_scope_t) -> Self {
        match val {
            v if v == wfa::alignment_scope_t_compute_alignment => Self::Alignment,
            v if v == wfa::alignment_scope_t_compute_score => Self::ComputeScore,
            _ => Self::Undefined,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlignmentSpan {
    End2End,
    EndsFree {
        pattern_begin_free: std::os::raw::c_int,
        pattern_end_free: std::os::raw::c_int,
        text_begin_free: std::os::raw::c_int,
        text_end_free: std::os::raw::c_int,
    },
    Undefined,
}

impl AlignmentSpan {
    pub fn from_form(form: wfa::alignment_form_t) -> Self {
        match form.span {
            v if v == wfa::alignment_span_t_alignment_end2end => Self::End2End,
            v if v == wfa::alignment_span_t_alignment_endsfree => Self::EndsFree {
                pattern_begin_free: form.pattern_begin_free,
                pattern_end_free: form.pattern_end_free,
                text_begin_free: form.text_begin_free,
                text_end_free: form.text_end_free,
            },
            _ => Self::Undefined,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MemoryMode {
    High,
    Medium,
    Low,
    Ultralow,
    Undefined,
}

impl MemoryMode {
    pub fn from_value(val: u32) -> Self {
        match val {
            v if v == wfa::wavefront_memory_t_wavefront_memory_high => Self::High,
            v if v == wfa::wavefront_memory_t_wavefront_memory_med => Self::Medium,
            v if v == wfa::wavefront_memory_t_wavefront_memory_low => Self::Low,
            v if v == wfa::wavefront_memory_t_wavefront_memory_ultralow => Self::Ultralow,
            _ => Self::Undefined,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlignmentStatus {
    Completed,
    Partial,
    MaxStepsReached,
    OOM,
    Unattainable,
    Undefined,
}

impl From<std::os::raw::c_int> for AlignmentStatus {
    fn from(value: std::os::raw::c_int) -> Self {
        match value {
            0 => AlignmentStatus::Completed,
            1 => AlignmentStatus::Partial,
            -100 => AlignmentStatus::MaxStepsReached,
            -200 => AlignmentStatus::OOM,
            -300 => AlignmentStatus::Unattainable,
            _ => AlignmentStatus::Undefined,
        }
    }
}

pub struct AffineWavefronts {
    wf_aligner: *mut wfa::wavefront_aligner_t,
}

impl Clone for AffineWavefronts {
    fn clone(&self) -> Self {
        Self {
            wf_aligner: self.wf_aligner,
        }
    }
}

impl Default for AffineWavefronts {
    fn default() -> Self {
        Self {
            // null pointer means wavefront_aligner_new will use default attributes.
            wf_aligner: unsafe { wfa::wavefront_aligner_new(core::ptr::null_mut()) },
        }
    }
}

impl Drop for AffineWavefronts {
    fn drop(&mut self) {
        unsafe {
            wfa::wavefront_aligner_delete(self.wf_aligner);
        }
    }
}

impl AffineWavefronts {
    pub fn aligner_mut(&mut self) -> *mut wfa::wavefront_aligner_t {
        self.wf_aligner
    }

    pub fn aligner(&self) -> *const wfa::wavefront_aligner_t {
        self.wf_aligner
    }

    fn new_aligner_edit(heuristic: Option<&HeuristicStrategy>) -> Self {
        unsafe {
            // Create attributes and set defaults (see https://github.com/smarco/WFA2-lib/blob/2ec2891/wavefront/wavefront_attributes.c#L38)
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance mode (includes distance metric and penalties)
            Self::set_distance_attr(&mut attributes, &Distance::Edit);

            // Set memory mode
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high; // wavefront_memory_t_wavefront_memory_ultralow does not work properly!

            // Configure heuristic before creating aligner
            Self::set_heuristic_attr(&mut attributes, heuristic);

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    fn new_aligner_gap_affine(
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
        heuristic: Option<&HeuristicStrategy>,
    ) -> Self {
        unsafe {
            // Create attributes and set defaults
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance mode (includes distance metric and penalties)
            Self::set_distance_attr(
                &mut attributes,
                &Distance::GapAffine {
                    mismatch,
                    gap_opening,
                    gap_extension,
                },
            );

            // Set memory mode
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high; // wavefront_memory_t_wavefront_memory_ultralow does not work properly!

            // Configure heuristic before creating aligner
            Self::set_heuristic_attr(&mut attributes, heuristic);

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    fn new_aligner_gap_affine2p(
        mismatch: i32,
        gap_opening1: i32,
        gap_extension1: i32,
        gap_opening2: i32,
        gap_extension2: i32,
        heuristic: Option<&HeuristicStrategy>,
    ) -> Self {
        unsafe {
            // Create attributes and set defaults (see https://github.com/smarco/WFA2-lib/blob/2ec2891/wavefront/wavefront_attributes.c#L38)
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance mode (includes distance metric and penalties)
            Self::set_distance_attr(
                &mut attributes,
                &Distance::GapAffine2p {
                    mismatch,
                    gap_opening1,
                    gap_extension1,
                    gap_opening2,
                    gap_extension2,
                },
            );

            // Set memory mode
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high; // wavefront_memory_t_wavefront_memory_ultralow does not work properly!

            // Configure heuristic before creating aligner
            Self::set_heuristic_attr(&mut attributes, heuristic);

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    /// Align two sequences and return the alignment status.
    pub fn align(&self, a: &[u8], b: &[u8]) -> AlignmentStatus {
        unsafe {
            let a = slice::from_raw_parts(a.as_ptr() as *const i8, a.len());
            let b = slice::from_raw_parts(b.as_ptr() as *const i8, b.len());

            let alignment_status: AlignmentStatus = wfa::wavefront_align(
                self.wf_aligner,
                a.as_ptr(),
                a.len() as i32,
                b.as_ptr(),
                b.len() as i32,
            )
            .into();

            alignment_status
        }
    }

    /// Returns the CIGAR string from the last alignment.
    pub fn cigar(&self) -> &[u8] {
        unsafe {
            let cigar = (*self.wf_aligner).cigar;
            let ops = (*cigar).operations;
            let begin_offset = (*cigar).begin_offset;
            let end_offset = (*cigar).end_offset;
            let length = end_offset - begin_offset;

            let cigar_slice: &[u8] = std::slice::from_raw_parts(
                (ops as *const u8).add(begin_offset as usize),
                length.try_into().unwrap(),
            );
            cigar_slice
        }
    }

    /// Returns the alignment score from the last alignment.
    pub fn score(&self) -> i32 {
        unsafe {
            let cigar = (*self.wf_aligner).cigar;
            (*cigar).score
        }
    }

    /// Reclaims any extra buffers the underlying WFA aligner grew during the last run.
    pub fn clear(&mut self) {
        unsafe {
            wfa::wavefront_aligner_reap(self.wf_aligner);
        }
    }

    /// Report the size of the underlying WFA aligner in bytes.
    pub fn get_aligner_size(&self) -> u64 {
        unsafe { wfa::wavefront_aligner_get_size(self.wf_aligner) }
    }

    fn set_distance_attr(attributes: &mut wfa::wavefront_aligner_attr_t, mode: &Distance) {
        match mode {
            Distance::Edit => {
                attributes.distance_metric = wfa::distance_metric_t_edit;
            }
            Distance::GapAffine {
                mismatch,
                gap_opening,
                gap_extension,
            } => {
                attributes.distance_metric = wfa::distance_metric_t_gap_affine;
                attributes.affine_penalties.mismatch = *mismatch;
                attributes.affine_penalties.gap_opening = *gap_opening;
                attributes.affine_penalties.gap_extension = *gap_extension;
            }
            Distance::GapAffine2p {
                mismatch,
                gap_opening1,
                gap_extension1,
                gap_opening2,
                gap_extension2,
            } => {
                attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
                attributes.affine2p_penalties.mismatch = *mismatch;
                attributes.affine2p_penalties.gap_opening1 = *gap_opening1;
                attributes.affine2p_penalties.gap_extension1 = *gap_extension1;
                attributes.affine2p_penalties.gap_opening2 = *gap_opening2;
                attributes.affine2p_penalties.gap_extension2 = *gap_extension2;
            }
        }
    }

    pub fn get_distance(&self) -> Distance {
        unsafe {
            let aligner = *self.aligner();
            let metric = aligner.penalties.distance_metric;

            match metric {
                wfa::distance_metric_t_edit => Distance::Edit,
                wfa::distance_metric_t_gap_affine => Distance::GapAffine {
                    mismatch: aligner.penalties.mismatch,
                    gap_opening: aligner.penalties.gap_opening1,
                    gap_extension: aligner.penalties.gap_extension1,
                },
                wfa::distance_metric_t_gap_affine_2p => Distance::GapAffine2p {
                    mismatch: aligner.penalties.mismatch,
                    gap_opening1: aligner.penalties.gap_opening1,
                    gap_extension1: aligner.penalties.gap_extension1,
                    gap_opening2: aligner.penalties.gap_opening2,
                    gap_extension2: aligner.penalties.gap_extension2,
                },
                _ => Distance::Edit, // Default fallback
            }
        }
    }

    fn set_heuristic_attr(
        attributes: &mut wfa::wavefront_aligner_attr_t,
        heuristic: Option<&HeuristicStrategy>,
    ) {
        match heuristic {
            Some(HeuristicStrategy::BandedStatic {
                band_min_k,
                band_max_k,
            }) => {
                attributes.heuristic.strategy =
                    wfa::wf_heuristic_strategy_wf_heuristic_banded_static;
                attributes.heuristic.min_k = *band_min_k;
                attributes.heuristic.max_k = *band_max_k;
            }
            Some(HeuristicStrategy::BandedAdaptive {
                band_min_k,
                band_max_k,
                score_steps,
            }) => {
                attributes.heuristic.strategy =
                    wfa::wf_heuristic_strategy_wf_heuristic_banded_adaptive;
                attributes.heuristic.min_k = *band_min_k;
                attributes.heuristic.max_k = *band_max_k;
                attributes.heuristic.steps_between_cutoffs = *score_steps;
            }
            Some(HeuristicStrategy::WFAdaptive {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            }) => {
                attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_wfadaptive;
                attributes.heuristic.min_wavefront_length = *min_wavefront_length;
                attributes.heuristic.max_distance_threshold = *max_distance_threshold;
                attributes.heuristic.steps_between_cutoffs = *score_steps;
            }
            Some(HeuristicStrategy::XDrop { xdrop, score_steps }) => {
                attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_xdrop;
                attributes.heuristic.xdrop = *xdrop;
                attributes.heuristic.steps_between_cutoffs = *score_steps;
            }
            Some(HeuristicStrategy::ZDrop { zdrop, score_steps }) => {
                attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_zdrop;
                attributes.heuristic.zdrop = *zdrop;
                attributes.heuristic.steps_between_cutoffs = *score_steps;
            }
            Some(HeuristicStrategy::WFMash {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            }) => {
                attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_wfmash;
                attributes.heuristic.min_wavefront_length = *min_wavefront_length;
                attributes.heuristic.max_distance_threshold = *max_distance_threshold;
                attributes.heuristic.steps_between_cutoffs = *score_steps;
            }
            Some(HeuristicStrategy::None) | _ => {
                attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;
            }
        }
    }

    /// Update heuristic on an already-created aligner
    pub fn set_heuristic(&mut self, heuristic: Option<&HeuristicStrategy>) {
        unsafe {
            match heuristic {
                Some(HeuristicStrategy::BandedStatic { band_min_k, band_max_k }) => {
                    wfa::wavefront_aligner_set_heuristic_banded_static(
                        self.wf_aligner,
                        *band_min_k,
                        *band_max_k,
                    );
                }
                Some(HeuristicStrategy::BandedAdaptive { band_min_k, band_max_k, score_steps }) => {
                    wfa::wavefront_aligner_set_heuristic_banded_adaptive(
                        self.wf_aligner,
                        *band_min_k,
                        *band_max_k,
                        *score_steps,
                    );
                }
                Some(HeuristicStrategy::WFAdaptive { min_wavefront_length, max_distance_threshold, score_steps }) => {
                    wfa::wavefront_aligner_set_heuristic_wfadaptive(
                        self.wf_aligner,
                        *min_wavefront_length,
                        *max_distance_threshold,
                        *score_steps,
                    );
                }
                Some(HeuristicStrategy::XDrop { xdrop, score_steps }) => {
                    wfa::wavefront_aligner_set_heuristic_xdrop(
                        self.wf_aligner,
                        *xdrop,
                        *score_steps,
                    );
                }
                Some(HeuristicStrategy::ZDrop { zdrop, score_steps }) => {
                    wfa::wavefront_aligner_set_heuristic_zdrop(
                        self.wf_aligner,
                        *zdrop,
                        *score_steps,
                    );
                }
                Some(HeuristicStrategy::WFMash { min_wavefront_length, max_distance_threshold, score_steps }) => {
                    wfa::wavefront_aligner_set_heuristic_wfmash(
                        self.wf_aligner,
                        *min_wavefront_length,
                        *max_distance_threshold,
                        *score_steps,
                    );
                }
                Some(HeuristicStrategy::None) | None => {
                    wfa::wavefront_aligner_set_heuristic_none(self.wf_aligner);
                }
            }
        }
    }

    pub fn get_heuristics(&self) -> Vec<HeuristicStrategy> {
        let mut hs = Vec::new();
        let heuristic = unsafe { *self.wf_aligner }.heuristic;
        let strategy = heuristic.strategy;

        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_zdrop > 0 {
            hs.push(HeuristicStrategy::ZDrop {
                zdrop: heuristic.zdrop,
                score_steps: heuristic.steps_between_cutoffs,
            });
        }
        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_xdrop > 0 {
            hs.push(HeuristicStrategy::XDrop {
                xdrop: heuristic.zdrop,
                score_steps: heuristic.steps_between_cutoffs,
            });
        }
        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_banded_adaptive > 0 {
            hs.push(HeuristicStrategy::BandedAdaptive {
                band_min_k: heuristic.min_k,
                band_max_k: heuristic.max_k,
                score_steps: heuristic.steps_between_cutoffs,
            });
        }
        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_banded_static > 0 {
            hs.push(HeuristicStrategy::BandedStatic {
                band_min_k: heuristic.min_k,
                band_max_k: heuristic.max_k,
            });
        }
        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_wfadaptive > 0 {
            hs.push(HeuristicStrategy::WFAdaptive {
                min_wavefront_length: heuristic.min_wavefront_length,
                max_distance_threshold: heuristic.max_distance_threshold,
                score_steps: heuristic.steps_between_cutoffs,
            });
        }
        if strategy & wfa::wf_heuristic_strategy_wf_heuristic_wfmash > 0 {
            hs.push(HeuristicStrategy::WFMash {
                min_wavefront_length: heuristic.min_wavefront_length,
                max_distance_threshold: heuristic.max_distance_threshold,
                score_steps: heuristic.steps_between_cutoffs,
            });
        }
        hs
    }

    pub fn get_alignment_scope(&self) -> AlignmentScope {
        let a = unsafe { *self.wf_aligner };
        AlignmentScope::from_scope(a.alignment_scope)
    }

    pub fn get_alignment_span(&self) -> AlignmentSpan {
        let form = unsafe { *self.aligner() }.alignment_form;
        AlignmentSpan::from_form(form)
    }

    pub fn get_memory_mode(&self) -> MemoryMode {
        let a = unsafe { *self.aligner() };
        MemoryMode::from_value(a.memory_mode)
    }
}
