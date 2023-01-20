use std::mem::MaybeUninit;
use world_sys::{InitializeCheapTrickOption, InitializeD4COption, InitializeHarvestOption};

/// Struct for CheapTrick
pub struct CheapTrickOption(world_sys::CheapTrickOption);

impl CheapTrickOption {
    /// f0_floor and fs are used to determine fft_size;
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/src/cheaptrick.cpp#L234>
    pub fn new(fs: i32) -> CheapTrickOption {
        let option = unsafe {
            let mut option = MaybeUninit::uninit();
            InitializeCheapTrickOption(fs, option.as_mut_ptr());
            option.assume_init()
        };
        CheapTrickOption(option)
    }

    /// q1 is the parameter used for the spectral recovery.
    ///
    /// Since The parameter is optimized, you don't need to change the parameter.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/src/cheaptrick.cpp#L231-L232>
    pub fn q1(&self) -> f64 {
        self.0.q1
    }

    /// f0_floor and fs are used to determine fft_size;
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/src/cheaptrick.cpp#L234>
    pub fn f0_floor(&self) -> f64 {
        self.0.f0_floor
    }

    /// We strongly recommend not to change this value unless you have enough
    /// knowledge of the signal processing in CheapTrick.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/src/cheaptrick.cpp#L235-L236>
    pub fn set_f0_floor(&mut self, f0_floor: f64) {
        self.0.f0_floor = f0_floor;
    }

    pub fn fft_size(&self) -> i32 {
        self.0.fft_size
    }

    /// We strongly recommend not to change this value unless you have enough
    /// knowledge of the signal processing in CheapTrick.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/src/cheaptrick.cpp#L235-L236>
    pub fn set_fft_size(&mut self, fft_size: i32) {
        self.0.fft_size = fft_size;
    }

    pub(crate) fn as_ptr(&self) -> *const world_sys::CheapTrickOption {
        &self.0
    }
}

/// Struct for D4C
pub struct D4COption(world_sys::D4COption);

impl D4COption {
    pub fn new() -> D4COption {
        let option = unsafe {
            let mut option = MaybeUninit::uninit();
            InitializeD4COption(option.as_mut_ptr());
            option.assume_init()
        };
        D4COption(option)
    }

    /// It is used to determine the aperiodicity in whole frequency band.
    /// D4C identifies whether the frame is voiced segment even if it had an F0.
    /// If the estimated value falls below the threshold,
    /// the aperiodicity in whole frequency band will set to 1.0.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/test/test.cpp#L201-L204>
    pub fn threshold(&self) -> f64 {
        self.0.threshold
    }

    /// If you want to use the conventional D4C, please set the threshold to 0.0.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/test/test.cpp#L205>
    pub fn set_threshold(&mut self, threshold: f64) {
        self.0.threshold = threshold;
    }

    pub(crate) fn as_ptr(&self) -> *const world_sys::D4COption {
        &self.0
    }
}

impl Default for D4COption {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct for Harvest
pub struct HarvestOption(world_sys::HarvestOption);

impl HarvestOption {
    pub fn new() -> HarvestOption {
        let option = unsafe {
            let mut option = MaybeUninit::uninit();
            InitializeHarvestOption(option.as_mut_ptr());
            option.assume_init()
        };
        HarvestOption(option)
    }

    pub fn f0_floor(&self) -> f64 {
        self.0.f0_floor
    }

    /// You can set the f0_floor below world::kFloorF0.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/test/test.cpp#L144>
    pub fn set_f0_floor(&mut self, f0_floor: f64) {
        self.0.f0_floor = f0_floor;
    }

    pub fn f0_ceil(&self) -> f64 {
        self.0.f0_ceil
    }

    pub fn set_f0_ceil(&mut self, f0_ceil: f64) {
        self.0.f0_ceil = f0_ceil;
    }

    pub fn frame_period(&self) -> f64 {
        self.0.frame_period
    }

    /// You can change the frame period.
    /// But the estimation is carried out with 1-ms frame shift.
    ///
    /// original: <https://github.com/mmorise/World/tree/v1.0.0/test/test.cpp#L140-L141>
    pub fn set_frame_period(&mut self, frame_period: f64) {
        self.0.frame_period = frame_period;
    }

    pub(crate) fn as_ptr(&self) -> *const world_sys::HarvestOption {
        &self.0
    }
}

impl Default for HarvestOption {
    fn default() -> Self {
        Self::new()
    }
}
