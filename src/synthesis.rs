use crate::spectrogram_like::SpectrogramLike;
use std::collections::VecDeque;
use std::mem::MaybeUninit;
use std::slice;
use world_sys::{AddParameters, DestroySynthesizer, InitializeSynthesizer, IsLocked, RefreshSynthesizer, Synthesis, Synthesis2, WorldSynthesizer};

pub enum SynthesisError {
    DifferentSizeInput,
    TooLargeValue,
    InvalidFFTSize,
}

pub fn synthesis_to(f0: &[f64], spectrogram: &SpectrogramLike<f64>, aperiodicity: &SpectrogramLike<f64>, fft_size: Option<i32>, frame_period: f64, fs: i32, out: &mut [f64]) -> Result<(), SynthesisError> {
    if f0.len() != spectrogram.time_axis_size() || spectrogram.time_axis_size() != aperiodicity.time_axis_size() || spectrogram.frequency_axis_size() != aperiodicity.frequency_axis_size() {
        return Err(SynthesisError::DifferentSizeInput);
    }
    if out.len() > i32::MAX as usize || f0.len() > i32::MAX as usize {
        return Err(SynthesisError::TooLargeValue);
    }
    let fft_size = fft_size.ok_or(()).or_else(|_| ((spectrogram.frequency_axis_size() - 1) * 2).try_into().map_err(|_| SynthesisError::TooLargeValue))?;
    if (fft_size / 2 + 1) as usize != spectrogram.frequency_axis_size() {
        return Err(SynthesisError::InvalidFFTSize);
    }
    unsafe { Synthesis(f0.as_ptr(), f0.len() as i32, spectrogram.as_ptr(), aperiodicity.as_ptr(), fft_size, frame_period, fs, out.len() as i32, out.as_mut_ptr()) }
    Ok(())
}

pub fn synthesis(f0: &[f64], spectrogram: &SpectrogramLike<f64>, aperiodicity: &SpectrogramLike<f64>, fft_size: Option<i32>, frame_period: f64, fs: i32) -> Result<Vec<f64>, SynthesisError> {
    let out_len = (f0.len() as f64 * frame_period * fs as f64 / 1000.).floor() as usize;
    let mut out = vec![0.; out_len];
    synthesis_to(f0, spectrogram, aperiodicity, fft_size, frame_period, fs, &mut out).map(move |_| out)
}

pub struct Synthesizer {
    synthesizer: WorldSynthesizer,
    queue: VecDeque<f64>,
}

impl Synthesizer {
    pub fn new(fs: i32, frame_period: f64, fft_size: i32) -> Synthesizer {
        let synthesizer = unsafe {
            let mut synthesizer = MaybeUninit::uninit();
            InitializeSynthesizer(fs, frame_period, fft_size, 128, 1, synthesizer.as_mut_ptr());
            synthesizer.assume_init()
        };
        Synthesizer { synthesizer, queue: VecDeque::new() }
    }

    pub fn add(&mut self, f0: &mut [f64], spectrogram: &mut SpectrogramLike<f64>, aperiodicity: &mut SpectrogramLike<f64>) -> Result<(), SynthesisError> {
        if f0.len() != spectrogram.time_axis_size() || spectrogram.time_axis_size() != aperiodicity.time_axis_size() || spectrogram.frequency_axis_size() != aperiodicity.frequency_axis_size() {
            return Err(SynthesisError::DifferentSizeInput);
        }
        if f0.len() > i32::MAX as usize {
            return Err(SynthesisError::TooLargeValue);
        }
        if (self.synthesizer.fft_size / 2 + 1) as usize != spectrogram.frequency_axis_size() {
            return Err(SynthesisError::InvalidFFTSize);
        }
        while unsafe { AddParameters(f0.as_mut_ptr(), f0.len() as i32, spectrogram.as_mut_ptr(), aperiodicity.as_mut_ptr(), &mut self.synthesizer) } == 0 {
            unsafe {
                if Synthesis2(&mut self.synthesizer) != 0 {
                    self.queue.extend(slice::from_raw_parts(self.synthesizer.buffer, self.synthesizer.buffer_size as usize).iter().copied());
                }
                if IsLocked(&mut self.synthesizer) != 0 {
                    RefreshSynthesizer(&mut self.synthesizer);
                }
            }
        }
        unsafe {
            while Synthesis2(&mut self.synthesizer) != 0 {
                self.queue.extend(slice::from_raw_parts(self.synthesizer.buffer, self.synthesizer.buffer_size as usize).iter().copied());
            }
            if IsLocked(&mut self.synthesizer) != 0 {
                RefreshSynthesizer(&mut self.synthesizer);
            }
        }
        Ok(())
    }
}

impl Drop for Synthesizer {
    fn drop(&mut self) {
        unsafe { DestroySynthesizer(&mut self.synthesizer) }
    }
}
