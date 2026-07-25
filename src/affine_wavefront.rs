use wfa::wavefront_aligner_set_max_alignment_steps;

use crate::bindings::*;
use std::os::raw::c_char;

#[derive(Debug, Clone, PartialEq)]
pub enum DistanceMetric {
    Indel,
    Edit,
    GapAffine,
    GapAffine2p,
}

/// Backwards-compatible distance configuration used by older callers.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn create_aligner(
        &self,
        heuristic: Option<&HeuristicStrategy>,
        memory_mode: Option<&MemoryMode>,
    ) -> AffineWavefronts {
        let mode = memory_mode.cloned().unwrap_or(MemoryMode::High);
        let mut aligner = match *self {
            Self::Edit => AffineWavefronts::with_edit_and_memory_mode(mode),
            Self::GapAffine {
                mismatch,
                gap_opening,
                gap_extension,
            } => AffineWavefronts::with_penalties_and_memory_mode(
                0,
                mismatch,
                gap_opening,
                gap_extension,
                mode,
            ),
            Self::GapAffine2p {
                mismatch,
                gap_opening1,
                gap_extension1,
                gap_opening2,
                gap_extension2,
            } => AffineWavefronts::with_penalties_affine2p_and_memory_mode(
                0,
                mismatch,
                gap_opening1,
                gap_extension1,
                gap_opening2,
                gap_extension2,
                mode,
            ),
        };

        aligner.set_heuristic(heuristic);
        aligner
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Edit => 0,
            Self::GapAffine { .. } => 1,
            Self::GapAffine2p { .. } => 2,
        }
    }

    pub fn from_u8(code: u8) -> Result<Self, String> {
        match code {
            0 => Ok(Self::Edit),
            1 => Ok(Self::GapAffine {
                mismatch: 0,
                gap_opening: 0,
                gap_extension: 0,
            }),
            2 => Ok(Self::GapAffine2p {
                mismatch: 0,
                gap_opening1: 0,
                gap_extension1: 0,
                gap_opening2: 0,
                gap_extension2: 0,
            }),
            _ => Err(format!("Invalid distance code: {code}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn to_wfa_value(&self) -> u32 {
        match self {
            Self::High => wfa::wavefront_memory_t_wavefront_memory_high,
            Self::Medium => wfa::wavefront_memory_t_wavefront_memory_med,
            Self::Low => wfa::wavefront_memory_t_wavefront_memory_low,
            Self::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
            Self::Undefined => wfa::wavefront_memory_t_wavefront_memory_high,
        }
    }
}

pub trait HeuristicArg<'a> {
    fn into_heuristic(self) -> Option<&'a HeuristicStrategy>;
}

impl<'a> HeuristicArg<'a> for &'a HeuristicStrategy {
    fn into_heuristic(self) -> Option<&'a HeuristicStrategy> {
        Some(self)
    }
}

impl<'a> HeuristicArg<'a> for Option<&'a HeuristicStrategy> {
    fn into_heuristic(self) -> Option<&'a HeuristicStrategy> {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        // Default to high memory mode for backward compatibility
        Self::with_penalties_and_memory_mode(
            match_,
            mismatch,
            gap_opening,
            gap_extension,
            MemoryMode::High,
        )
    }

    pub fn with_penalties_and_memory_mode(
        match_: i32,
        mismatch: i32,
        gap_opening: i32,
        gap_extension: i32,
        memory_mode: MemoryMode,
    ) -> Self {
        unsafe {
            // Create attributes and set defaults
            let mut attributes = wfa::wavefront_aligner_attr_default;

            // Set distance metric
            attributes.distance_metric = wfa::distance_metric_t_gap_affine;

            // Set penalties
            attributes.affine_penalties.match_ = match_;
            attributes.affine_penalties.mismatch = mismatch;
            attributes.affine_penalties.gap_opening = gap_opening;
            attributes.affine_penalties.gap_extension = gap_extension;

            // Set memory mode based on parameter
            attributes.memory_mode = match memory_mode {
                MemoryMode::High => wfa::wavefront_memory_t_wavefront_memory_high,
                MemoryMode::Medium => wfa::wavefront_memory_t_wavefront_memory_med,
                MemoryMode::Low => wfa::wavefront_memory_t_wavefront_memory_low,
                MemoryMode::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
                MemoryMode::Undefined => panic!("Cannot create aligner with undefined memory mode"),
            };

            // Disable heuristic
            attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;

            // Create aligner with attributes
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
    }

    pub fn with_edit_and_memory_mode(memory_mode: MemoryMode) -> Self {
        unsafe {
            let mut attributes = wfa::wavefront_aligner_attr_default;

            attributes.distance_metric = wfa::distance_metric_t_edit;
            attributes.memory_mode = memory_mode.to_wfa_value();
            attributes.heuristic.strategy = wfa::wf_heuristic_strategy_wf_heuristic_none;

            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);

            Self { wf_aligner }
        }
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
        // Default to high memory mode for backward compatibility
        Self::with_penalties_affine2p_and_memory_mode(
            match_,
            mismatch,
            gap_opening1,
            gap_extension1,
            gap_opening2,
            gap_extension2,
            MemoryMode::High,
        )
    }

    pub fn with_penalties_affine2p_and_memory_mode(
        match_: i32,
        mismatch: i32,
        gap_opening1: i32,
        gap_extension1: i32,
        gap_opening2: i32,
        gap_extension2: i32,
        memory_mode: MemoryMode,
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

            // Set memory mode based on parameter
            attributes.memory_mode = match memory_mode {
                MemoryMode::High => wfa::wavefront_memory_t_wavefront_memory_high,
                MemoryMode::Medium => wfa::wavefront_memory_t_wavefront_memory_med,
                MemoryMode::Low => wfa::wavefront_memory_t_wavefront_memory_low,
                MemoryMode::Ultralow => wfa::wavefront_memory_t_wavefront_memory_ultralow,
                MemoryMode::Undefined => panic!("Cannot create aligner with undefined memory mode"),
            };

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

    pub fn get_distance(&self) -> Distance {
        unsafe {
            let penalties = (*self.wf_aligner).penalties;
            match penalties.distance_metric {
                m if m == wfa::distance_metric_t_edit => Distance::Edit,
                m if m == wfa::distance_metric_t_gap_affine => Distance::GapAffine {
                    mismatch: penalties.mismatch,
                    gap_opening: penalties.gap_opening1,
                    gap_extension: penalties.gap_extension1,
                },
                m if m == wfa::distance_metric_t_gap_affine_2p => Distance::GapAffine2p {
                    mismatch: penalties.mismatch,
                    gap_opening1: penalties.gap_opening1,
                    gap_extension1: penalties.gap_extension1,
                    gap_opening2: penalties.gap_opening2,
                    gap_extension2: penalties.gap_extension2,
                },
                _ => Distance::Edit,
            }
        }
    }

    pub fn set_heuristic<'a, H>(&mut self, heuristic: H)
    where
        H: HeuristicArg<'a>,
    {
        match heuristic.into_heuristic() {
            None | Some(HeuristicStrategy::None) => unsafe {
                wfa::wavefront_aligner_set_heuristic_none(self.wf_aligner)
            },
            Some(HeuristicStrategy::BandedStatic {
                band_min_k,
                band_max_k,
            }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_banded_static(
                    self.wf_aligner,
                    *band_min_k,
                    *band_max_k,
                )
            },
            Some(HeuristicStrategy::BandedAdaptive {
                band_min_k,
                band_max_k,
                score_steps,
            }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_banded_adaptive(
                    self.wf_aligner,
                    *band_min_k,
                    *band_max_k,
                    *score_steps,
                )
            },
            Some(HeuristicStrategy::WFAdaptive {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_wfadaptive(
                    self.wf_aligner,
                    *min_wavefront_length,
                    *max_distance_threshold,
                    *score_steps,
                )
            },
            Some(HeuristicStrategy::XDrop { xdrop, score_steps }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_xdrop(self.wf_aligner, *xdrop, *score_steps)
            },
            Some(HeuristicStrategy::ZDrop { zdrop, score_steps }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_zdrop(self.wf_aligner, *zdrop, *score_steps)
            },
            Some(HeuristicStrategy::WFMash {
                min_wavefront_length,
                max_distance_threshold,
                score_steps,
            }) => unsafe {
                wfa::wavefront_aligner_set_heuristic_wfmash(
                    self.wf_aligner,
                    *min_wavefront_length,
                    *max_distance_threshold,
                    *score_steps,
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

    // REMOVED: set_memory_mode() - This method was removed because it only changes the 
    // memory_mode field but does NOT reconfigure the underlying WFA2 aligner. 
    // Memory mode must be set at creation time using the appropriate constructors:
    // - with_penalties_and_memory_mode()
    // - with_penalties_affine2p_and_memory_mode()
    // - AffineWavefrontsBuilder::new().memory_mode(...).build()

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
            // c_char is i8 on x86_64 but u8 on aarch64-linux, so cast to it by name.
            let alignment_status: AlignmentStatus = wfa::wavefront_align(
                self.wf_aligner,
                a.as_ptr() as *const c_char,
                a.len() as i32,
                b.as_ptr() as *const c_char,
                b.len() as i32,
            )
            .into();

            alignment_status
        }
    }

    // Convenient constructor for bi-WFA with ultralow memory
    pub fn new_ultralow() -> Self {
        Self::with_penalties_affine2p_and_memory_mode(
            0,  // match
            4,  // mismatch
            6,  // gap_opening1
            2,  // gap_extension1
            12, // gap_opening2
            1,  // gap_extension2
            MemoryMode::Ultralow,
        )
    }
}

// Builder pattern for more complex configurations
pub struct AffineWavefrontsBuilder {
    distance_metric: DistanceMetric,
    match_score: i32,
    mismatch_penalty: i32,
    gap_opening1: i32,
    gap_extension1: i32,
    gap_opening2: Option<i32>,
    gap_extension2: Option<i32>,
    memory_mode: MemoryMode,
    heuristic: HeuristicStrategy,
    alignment_scope: AlignmentScope,
}

impl Default for AffineWavefrontsBuilder {
    fn default() -> Self {
        Self {
            distance_metric: DistanceMetric::GapAffine,
            match_score: 0,
            mismatch_penalty: 4,
            gap_opening1: 6,
            gap_extension1: 2,
            gap_opening2: None,
            gap_extension2: None,
            memory_mode: MemoryMode::High,
            heuristic: HeuristicStrategy::None,
            alignment_scope: AlignmentScope::Alignment,
        }
    }
}

impl AffineWavefrontsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.distance_metric = metric;
        self
    }

    pub fn penalties(mut self, match_: i32, mismatch: i32, gap_open: i32, gap_ext: i32) -> Self {
        self.match_score = match_;
        self.mismatch_penalty = mismatch;
        self.gap_opening1 = gap_open;
        self.gap_extension1 = gap_ext;
        self
    }

    pub fn dual_affine_penalties(mut self, gap_open2: i32, gap_ext2: i32) -> Self {
        self.distance_metric = DistanceMetric::GapAffine2p;
        self.gap_opening2 = Some(gap_open2);
        self.gap_extension2 = Some(gap_ext2);
        self
    }

    pub fn memory_mode(mut self, mode: MemoryMode) -> Self {
        self.memory_mode = mode;
        self
    }

    pub fn heuristic(mut self, strategy: HeuristicStrategy) -> Self {
        self.heuristic = strategy;
        self
    }

    pub fn alignment_scope(mut self, scope: AlignmentScope) -> Self {
        self.alignment_scope = scope;
        self
    }

    pub fn build(self) -> AffineWavefronts {
        let mut aligner = match self.distance_metric {
            DistanceMetric::GapAffine => {
                AffineWavefronts::with_penalties_and_memory_mode(
                    self.match_score,
                    self.mismatch_penalty,
                    self.gap_opening1,
                    self.gap_extension1,
                    self.memory_mode,
                )
            }
            DistanceMetric::GapAffine2p => {
                AffineWavefronts::with_penalties_affine2p_and_memory_mode(
                    self.match_score,
                    self.mismatch_penalty,
                    self.gap_opening1,
                    self.gap_extension1,
                    self.gap_opening2.unwrap_or(12),
                    self.gap_extension2.unwrap_or(1),
                    self.memory_mode,
                )
            }
            _ => panic!("Distance metric {:?} not yet supported in builder", self.distance_metric),
        };

        aligner.set_heuristic(&self.heuristic);
        aligner.set_alignment_scope(self.alignment_scope);

        aligner
    }
}
