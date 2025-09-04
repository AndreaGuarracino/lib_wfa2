use wfa::wavefront_aligner_set_max_alignment_steps;

use crate::bindings::*;
use core::slice;

#[derive(Debug, Clone)]
pub enum DistanceMetric {
    Indel,
    Edit,
    GapAffine,
    GapAffine2p,
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

    pub fn set_penalties(
        &mut self,
        match_: i32,
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
    ) {
        unsafe {
            (*self.wf_aligner).penalties.match_ = match_;
            (*self.wf_aligner).penalties.mismatch = mismatch;
            (*self.wf_aligner).penalties.gap_opening1 = gap_opening;
            (*self.wf_aligner).penalties.gap_extension1 = gap_extension;
        }
    }

    pub fn with_penalties(
        match_: i32,
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
    ) -> Self {
        let s = Self {
            wf_aligner: unsafe { wfa::wavefront_aligner_new(core::ptr::null_mut()) },
        };
        unsafe {
            (*s.wf_aligner).penalties.match_ = match_;
            (*s.wf_aligner).penalties.mismatch = mismatch;
            (*s.wf_aligner).penalties.gap_opening1 = gap_opening;
            (*s.wf_aligner).penalties.gap_extension1 = gap_extension;
        }

        s
    }

    pub fn set_penalties_affine2p(
        &mut self,
        match_: i32,
        mismatch: i32,
        gap_opening1: i32,
        gap_extension1: i32,
        gap_opening2: i32,
        gap_extension2: i32,
    ) {
        unsafe {
            (*self.wf_aligner).penalties.match_ = match_;
            (*self.wf_aligner).penalties.mismatch = mismatch;
            (*self.wf_aligner).penalties.gap_opening1 = gap_opening1;
            (*self.wf_aligner).penalties.gap_extension1 = gap_extension1;
            (*self.wf_aligner).penalties.gap_opening2 = gap_opening2;
            (*self.wf_aligner).penalties.gap_extension2 = gap_extension2;
        }
    }

    pub fn with_penalties_affine2p(
        match_: i32,
        mismatch: i32,
        gap_opening1: i32,
        gap_extension1: i32,
        gap_opening2: i32,
        gap_extension2: i32,
    ) -> Self {
        unsafe {
            // Create attributes and set defaults (see https://github.com/smarco/WFA2-lib/blob/2ec2891/wavefront/wavefront_attributes.c#L38)
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance metric
            attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;

            // Set penalties
            attributes.affine2p_penalties.match_ = match_;
            attributes.affine2p_penalties.mismatch = mismatch;
            attributes.affine2p_penalties.gap_opening1 = gap_opening1;
            attributes.affine2p_penalties.gap_extension1 = gap_extension1;
            attributes.affine2p_penalties.gap_opening2 = gap_opening2;
            attributes.affine2p_penalties.gap_extension2 = gap_extension2;

            // Set memory mode
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high;

            // Disable heuristic
            attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    pub fn set_penalties_edit(
        &mut self,
        match_: i32,
        mismatch: i32,
        gap_opening1: i32,
    ) {
        unsafe {
            (*self.wf_aligner).penalties.match_ = match_;
            (*self.wf_aligner).penalties.mismatch = mismatch;
            (*self.wf_aligner).penalties.gap_opening1 = gap_opening1;
            (*self.wf_aligner).penalties.gap_extension1 = -1;
            (*self.wf_aligner).penalties.gap_opening2 = -1;
            (*self.wf_aligner).penalties.gap_extension2 = -1;
        }
    }

    pub fn with_penalties_edit(
        match_: i32,
        mismatch: i32,
        gap_opening1: i32,
    ) -> Self {
        unsafe {
            // Create attributes and set defaults (see https://github.com/smarco/WFA2-lib/blob/2ec2891/wavefront/wavefront_attributes.c#L38)
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance metric
            attributes.distance_metric = wfa::distance_metric_t_edit;

            // Set penalties
            attributes.affine2p_penalties.match_ = match_;
            attributes.affine2p_penalties.mismatch = mismatch;
            attributes.affine2p_penalties.gap_opening1 = gap_opening1;
            attributes.affine2p_penalties.gap_extension1 = -1;
            attributes.affine2p_penalties.gap_opening2 = -1;
            attributes.affine2p_penalties.gap_extension2 = -1;

            // Set memory mode
            attributes.memory_mode = wfa::wavefront_memory_t_wavefront_memory_high;

            // Disable heuristic
            attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    pub fn get_distance_metric(&self) -> DistanceMetric {
        unsafe {
            match (*self.wf_aligner).penalties.distance_metric {
                m if m == wfa::distance_metric_t_indel => DistanceMetric::Indel,
                m if m == wfa::distance_metric_t_edit => DistanceMetric::Edit,
                m if m == wfa::distance_metric_t_gap_affine => DistanceMetric::GapAffine,
                m if m == wfa::distance_metric_t_gap_affine_2p => DistanceMetric::GapAffine2p,
                _ => DistanceMetric::GapAffine, // Default to gap-affine
            }
        }
    }

    pub fn set_heuristic(&mut self, heuristic: &HeuristicStrategy) {
        match *heuristic {
            HeuristicStrategy::None => unsafe {
                wfa::wavefront_aligner_set_heuristic_none(self.wf_aligner)
            },
            HeuristicStrategy::BandedStatic {
                band_min_k,
                band_max_k,
            } => unsafe {
                wfa::wavefront_aligner_set_heuristic_banded_static(
                    self.wf_aligner,
                    band_min_k,
                    band_max_k,
                )
            },
            HeuristicStrategy::BandedAdaptive {
                band_min_k,
                band_max_k,
                score_steps,
            } => unsafe {
                wfa::wavefront_aligner_set_heuristic_banded_adaptive(
                    self.wf_aligner,
                    band_min_k,
                    band_max_k,
                    score_steps,
                )
            },
            HeuristicStrategy::WFAdaptive {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            } => unsafe {
                wfa::wavefront_aligner_set_heuristic_wfadaptive(
                    self.wf_aligner,
                    min_wavefront_length,
                    max_distance_threshold,
                    score_steps,
                )
            },
            HeuristicStrategy::XDrop { xdrop, score_steps } => unsafe {
                wfa::wavefront_aligner_set_heuristic_xdrop(self.wf_aligner, xdrop, score_steps)
            },
            HeuristicStrategy::ZDrop { zdrop, score_steps } => unsafe {
                wfa::wavefront_aligner_set_heuristic_zdrop(self.wf_aligner, zdrop, score_steps)
            },
            HeuristicStrategy::WFMash {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            } => unsafe {
                wfa::wavefront_aligner_set_heuristic_wfmash(
                    self.wf_aligner,
                    min_wavefront_length,
                    max_distance_threshold,
                    score_steps,
                )
            },
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

    pub fn set_alignment_scope(&mut self, scope: AlignmentScope) {
        (unsafe { *self.wf_aligner }).alignment_scope = match scope {
            AlignmentScope::ComputeScore => wfa::alignment_scope_t_compute_score,
            AlignmentScope::Alignment => wfa::alignment_scope_t_compute_alignment,
            AlignmentScope::Undefined => panic!("Cannot set an undefined scope"),
        }
    }

    pub fn get_alignment_scope(&self) -> AlignmentScope {
        let a = unsafe { *self.wf_aligner };
        AlignmentScope::from_scope(a.alignment_scope)
    }

    pub fn set_alignment_span(&mut self, span: AlignmentSpan) {
        let _form: &mut wfa::alignment_form_t = &mut (unsafe { *self.wf_aligner }).alignment_form;
        match span {
            AlignmentSpan::End2End => {
                unsafe { wfa::wavefront_aligner_set_alignment_end_to_end(self.wf_aligner) };
            }
            AlignmentSpan::EndsFree {
                pattern_begin_free,
                pattern_end_free,
                text_begin_free,
                text_end_free,
            } => {
                unsafe {
                    wfa::wavefront_aligner_set_alignment_free_ends(
                        self.wf_aligner,
                        pattern_begin_free,
                        pattern_end_free,
                        text_begin_free,
                        text_end_free,
                    )
                };
            }
            AlignmentSpan::Undefined => (),
        }
    }

    pub fn set_memory_mode(&mut self, mode: MemoryMode) {
        (unsafe { *self.wf_aligner }).memory_mode = match mode {
            MemoryMode::High => wfa::wavefront_memory_t_wavefront_memory_high,
            MemoryMode::Medium => wfa::wavefront_memory_t_wavefront_memory_med,
            MemoryMode::Low => wfa::wavefront_memory_t_wavefront_memory_low,
            MemoryMode::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
            MemoryMode::Undefined => panic!("Cannot set Undefined memory mode!"),
        }
    }

    pub fn get_memory_mode(&self) -> MemoryMode {
        let a = unsafe { *self.aligner() };
        MemoryMode::from_value(a.memory_mode)
    }

    pub fn get_alignment_span(&self) -> AlignmentSpan {
        let form = unsafe { *self.aligner() }.alignment_form;
        AlignmentSpan::from_form(form)
    }

    pub fn set_max_alignment_steps(&mut self, steps: i32) {
        unsafe {
            wavefront_aligner_set_max_alignment_steps(self.wf_aligner, steps);
        }
    }

    pub fn set_max_alignment_score(&mut self, score: i32) {
        self.set_max_alignment_steps(score);
    }

    pub fn get_max_alignment_steps(&self) -> i32 {
        let a = unsafe { *self.aligner() };
        a.system.max_alignment_steps
    }

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

    pub fn score(&self) -> i32 {
        unsafe {
            let cigar = (*self.wf_aligner).cigar;
            (*cigar).score
        }
    }

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
}
