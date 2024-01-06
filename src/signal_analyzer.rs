use crate::options::{CheapTrickOption, D4COption, HarvestOption};
use crate::spectrogram_like::SpectrogramLike;
use std::sync::OnceLock;
use world_sys::{CheapTrick, GetSamplesForHarvest, Harvest, D4C};

struct HarvestResult {
    f0: Box<[f64]>,
    temporal_positions: Box<[f64]>,
}

pub struct SignalAnalyzer {
    signal: Box<[f64]>,
    fs: i32,
    harvest_option: HarvestOption,
    cheaptrick_option: CheapTrickOption,
    d4c_option: D4COption,
    harvest_result: OnceLock<HarvestResult>,
    spectrogram: OnceLock<SpectrogramLike<f64>>,
    aperiodicity: OnceLock<SpectrogramLike<f64>>,
}

pub struct AnalyzeResult {
    pub signal: Box<[f64]>,
    pub f0: Option<Box<[f64]>>,
    pub spectrogram: Option<SpectrogramLike<f64>>,
    pub aperiodicity: Option<SpectrogramLike<f64>>,
}

pub struct SignalAnalyzerBuilder {
    fs: i32,
    harvest_option: HarvestOption,
    cheaptrick_option: CheapTrickOption,
    d4c_option: D4COption,
}

impl SignalAnalyzerBuilder {
    pub fn new(fs: u32) -> SignalAnalyzerBuilder {
        let fs = fs.try_into().unwrap();
        SignalAnalyzerBuilder {
            fs,
            harvest_option: HarvestOption::new(),
            cheaptrick_option: CheapTrickOption::new(fs),
            d4c_option: D4COption::new(),
        }
    }

    pub fn fs(&self) -> i32 {
        self.fs
    }

    pub fn harvest_option(&self) -> &HarvestOption {
        &self.harvest_option
    }

    pub fn harvest_option_mut(&mut self) -> &mut HarvestOption {
        &mut self.harvest_option
    }

    pub fn cheaptrick_option(&self) -> &CheapTrickOption {
        &self.cheaptrick_option
    }

    pub fn cheaptrick_option_mut(&mut self) -> &mut CheapTrickOption {
        &mut self.cheaptrick_option
    }

    pub fn d4c_option(&self) -> &D4COption {
        &self.d4c_option
    }

    pub fn d4c_option_mut(&mut self) -> &mut D4COption {
        &mut self.d4c_option
    }

    pub fn build(self, signal: Box<[f64]>) -> SignalAnalyzer {
        SignalAnalyzer::from_builder(self, signal)
    }
}

impl SignalAnalyzer {
    pub fn from_builder(builder: SignalAnalyzerBuilder, signal: Box<[f64]>) -> SignalAnalyzer {
        let SignalAnalyzerBuilder {
            fs,
            harvest_option,
            cheaptrick_option,
            d4c_option,
        } = builder;
        assert!(signal.len() <= i32::MAX as usize);
        SignalAnalyzer {
            signal,
            fs,
            harvest_option,
            cheaptrick_option,
            d4c_option,
            harvest_result: OnceLock::new(),
            spectrogram: OnceLock::new(),
            aperiodicity: OnceLock::new(),
        }
    }

    pub fn new(signal: Box<[f64]>, fs: u32) -> SignalAnalyzer {
        assert!(signal.len() <= i32::MAX as usize);
        let fs = fs.try_into().unwrap();
        SignalAnalyzer {
            signal,
            fs,
            harvest_option: HarvestOption::new(),
            cheaptrick_option: CheapTrickOption::new(fs),
            d4c_option: D4COption::new(),
            harvest_result: OnceLock::new(),
            spectrogram: OnceLock::new(),
            aperiodicity: OnceLock::new(),
        }
    }

    pub fn fs(&self) -> i32 {
        self.fs
    }

    pub fn harvest_option(&self) -> &HarvestOption {
        &self.harvest_option
    }

    pub fn cheaptrick_option(&self) -> &CheapTrickOption {
        &self.cheaptrick_option
    }

    pub fn d4c_option(&self) -> &D4COption {
        &self.d4c_option
    }

    fn harvest_result(&self) -> &HarvestResult {
        self.harvest_result.get_or_init(|| {
            let samples = unsafe { GetSamplesForHarvest(self.fs, self.signal.len() as i32, self.harvest_option.frame_period()) };
            let mut temporal_positions = vec![0.; samples as usize].into_boxed_slice();
            let mut f0 = vec![0.; samples as usize].into_boxed_slice();
            unsafe {
                Harvest(
                    self.signal.as_ptr(),
                    self.signal.len() as i32,
                    self.fs,
                    self.harvest_option.as_ptr(),
                    temporal_positions.as_mut_ptr(),
                    f0.as_mut_ptr(),
                );
            }
            HarvestResult { temporal_positions, f0 }
        })
    }

    pub fn f0(&self) -> &[f64] {
        &self.harvest_result().f0
    }

    pub fn temporal_positions(&self) -> &[f64] {
        &self.harvest_result().temporal_positions
    }

    pub fn spectrogram(&self) -> &SpectrogramLike<f64> {
        self.spectrogram.get_or_init(|| {
            let HarvestResult { f0, temporal_positions } = self.harvest_result();
            let mut spectrogram = SpectrogramLike::new(f0.len(), self.cheaptrick_option.fft_size() as usize / 2 + 1);
            unsafe {
                CheapTrick(
                    self.signal.as_ptr(),
                    self.signal.len() as i32,
                    self.fs,
                    temporal_positions.as_ptr(),
                    f0.as_ptr(),
                    f0.len() as i32,
                    self.cheaptrick_option.as_ptr(),
                    spectrogram.as_mut_ptr(),
                );
            }
            spectrogram
        })
    }

    pub fn aperiodicity(&self) -> &SpectrogramLike<f64> {
        self.aperiodicity.get_or_init(|| {
            let HarvestResult { f0, temporal_positions } = self.harvest_result();
            let mut aperiodicity = SpectrogramLike::new(f0.len(), self.cheaptrick_option.fft_size() as usize / 2 + 1);
            unsafe {
                D4C(
                    self.signal.as_ptr(),
                    self.signal.len() as i32,
                    self.fs,
                    temporal_positions.as_ptr(),
                    f0.as_ptr(),
                    f0.len() as i32,
                    self.cheaptrick_option.fft_size(),
                    self.d4c_option.as_ptr(),
                    aperiodicity.as_mut_ptr(),
                );
            }
            aperiodicity
        })
    }

    pub fn calc_all(&self) {
        self.spectrogram();
        self.aperiodicity();
    }

    pub fn into_result(self) -> AnalyzeResult {
        let SignalAnalyzer {
            signal,
            harvest_result,
            spectrogram,
            aperiodicity,
            ..
        } = self;
        AnalyzeResult {
            signal,
            f0: harvest_result.into_inner().map(|HarvestResult { f0, .. }| f0),
            spectrogram: spectrogram.into_inner(),
            aperiodicity: aperiodicity.into_inner(),
        }
    }
}
