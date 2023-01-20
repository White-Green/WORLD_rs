use crate::options::{CheapTrickOption, D4COption, HarvestOption};
use crate::spectrogram_like::SpectrogramLike;
use once_cell::sync::OnceCell;
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
    harvest_result: OnceCell<HarvestResult>,
    spectrogram: OnceCell<SpectrogramLike<f64>>,
    aperiodicity: OnceCell<SpectrogramLike<f64>>,
}

impl SignalAnalyzer {
    pub fn new(signal: Box<[f64]>, fs: u32) -> SignalAnalyzer {
        assert!(signal.len() <= i32::MAX as usize);
        let fs = fs.try_into().unwrap();
        SignalAnalyzer {
            signal,
            fs,
            harvest_option: HarvestOption::new(),
            cheaptrick_option: CheapTrickOption::new(fs),
            d4c_option: D4COption::new(),
            harvest_result: OnceCell::new(),
            spectrogram: OnceCell::new(),
            aperiodicity: OnceCell::new(),
        }
    }

    fn harvest_result(&self) -> &HarvestResult {
        self.harvest_result.get_or_init(|| {
            let samples = unsafe { GetSamplesForHarvest(self.fs, self.signal.len() as i32, self.harvest_option.frame_period()) };
            let mut temporal_positions = vec![0.; samples as usize].into_boxed_slice();
            let mut f0 = vec![0.; samples as usize].into_boxed_slice();
            unsafe {
                Harvest(self.signal.as_ptr(), self.signal.len() as i32, self.fs, self.harvest_option.as_ptr(), temporal_positions.as_mut_ptr(), f0.as_mut_ptr());
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
                CheapTrick(self.signal.as_ptr(), self.signal.len() as i32, self.fs, temporal_positions.as_ptr(), f0.as_ptr(), f0.len() as i32, self.cheaptrick_option.as_ptr(), spectrogram.as_mut_ptr());
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
}
