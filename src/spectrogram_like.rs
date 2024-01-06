pub struct SpectrogramLike<T> {
    all: Box<[T]>,
    lines: Box<[*mut T]>,
}

impl<T: Default + Copy> SpectrogramLike<T> {
    pub fn new(time_axis_size: usize, frequency_axis_size: usize) -> SpectrogramLike<T> {
        assert!(time_axis_size * frequency_axis_size > 0);
        let mut all = vec![T::default(); time_axis_size * frequency_axis_size].into_boxed_slice();
        let mut chunks_iter = all.chunks_exact_mut(frequency_axis_size);
        let lines = chunks_iter.by_ref().map(|slice| slice.as_mut_ptr()).collect::<Box<[_]>>();
        assert!(chunks_iter.into_remainder().is_empty());
        assert_eq!(lines.len(), time_axis_size);
        SpectrogramLike { all, lines }
    }
}

impl<T> SpectrogramLike<T> {
    pub fn time_axis_size(&self) -> usize {
        self.lines.len()
    }

    pub fn frequency_axis_size(&self) -> usize {
        self.all.len() / self.lines.len()
    }

    pub fn line(&self, line: usize) -> Option<&[T]> {
        let f = self.frequency_axis_size();
        self.all.get(line * f..(line + 1) * f)
    }

    pub fn line_mut(&mut self, line: usize) -> Option<&mut [T]> {
        let f = self.frequency_axis_size();
        self.all.get_mut(line * f..(line + 1) * f)
    }

    pub fn lines(&self) -> impl Iterator<Item = &[T]> + '_ {
        let f = self.frequency_axis_size();
        self.all.chunks(f)
    }

    pub fn lines_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        let f = self.frequency_axis_size();
        self.all.chunks_mut(f)
    }

    pub fn as_ptr(&self) -> *const *const T {
        self.lines.as_ptr() as *const *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut T {
        self.lines.as_mut_ptr()
    }
}

#[cfg(feature = "ndarray")]
mod ndarray {
    use ndarray::Array2;

    use super::SpectrogramLike;

    impl<T: Clone> From<Array2<T>> for SpectrogramLike<T> {
        fn from(arr: Array2<T>) -> Self {
            let mut arr = if arr.is_standard_layout() {
                arr
            } else {
                arr.as_standard_layout().into_owned()
            };

            let lines = arr.rows_mut().into_iter().map(|row| { row }.as_mut_ptr()).collect();

            let all = arr.into_raw_vec().into();

            Self { all, lines }
        }
    }

    impl<T> From<SpectrogramLike<T>> for Array2<T> {
        fn from(spectrogram_like: SpectrogramLike<T>) -> Self {
            let SpectrogramLike { all, lines } = spectrogram_like;
            assert!(all.len() % lines.len() == 0);
            let nrows = lines.len();
            let ncols = all.len() / lines.len();
            Self::from_shape_vec((nrows, ncols), all.into()).expect("should match")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrogram_like() {
        let mut spec = SpectrogramLike::<u32>::new(10, 5);
        assert_eq!(spec.time_axis_size(), 10);
        assert_eq!(spec.frequency_axis_size(), 5);
        spec.lines_mut()
            .enumerate()
            .for_each(|(i, line)| line.iter_mut().enumerate().for_each(|(j, item)| *item = (i * 5 + j) as u32));
        spec.lines()
            .enumerate()
            .for_each(|(i, line)| line.iter().enumerate().for_each(|(j, item)| assert_eq!(*item, (i * 5 + j) as u32)));
        let ptr = spec.as_mut_ptr();
        for i in 0..10 {
            for j in 0..5 {
                assert_eq!(unsafe { *(*ptr.offset(i)).offset(j) }, (i * 5 + j) as u32);
            }
        }
    }
}
