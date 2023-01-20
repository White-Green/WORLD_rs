pub struct SpectrogramLike<T> {
    all: Box<[T]>,
    lines: Box<[*mut T]>,
}

impl<T: Default + Copy> SpectrogramLike<T> {
    pub fn new(time_axis_size: usize, frequency_axis_size: usize) -> SpectrogramLike<T> {
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

    pub fn ptr(&mut self) -> *mut *mut T {
        self.lines.as_mut_ptr()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_spectrogram_like(){
        let mut spec = SpectrogramLike::<u32>::new(10,5);
        assert_eq!(spec.time_axis_size(), 10);
        assert_eq!(spec.frequency_axis_size(), 5);
        spec.lines_mut()
            .enumerate()
            .for_each(|(i, line)|line.iter_mut().enumerate().for_each(|(j, item)|*item=(i*5+j) as u32));
        spec.lines()
            .enumerate()
            .for_each(|(i, line)|line.iter().enumerate().for_each(|(j, item)| assert_eq!(*item, (i * 5 + j) as u32)));
        let ptr = spec.ptr();
        for i in 0..10{
            for j in 0..5{
                assert_eq!(unsafe { *(*ptr.offset(i)).offset(j) }, (i * 5 + j) as u32);
            }
        }
    }
}
