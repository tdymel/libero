pub struct ConstArr<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> ConstArr<T, N> {
    pub const fn into_inner(self) -> [T; N] {
        use std::{mem::ManuallyDrop, ptr};

        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&this.0) }
    }

    pub const fn append(self, value: T) -> ConstArr<T, { N + 1 }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let values = unsafe { ptr::read(&this.0) };
        let values = ManuallyDrop::new(values);
        let value = ManuallyDrop::new(value);

        let mut result = MaybeUninit::<[T; N + 1]>::uninit();
        let ptr = result.as_mut_ptr() as *mut T;

        let values = unsafe {
            ptr::copy_nonoverlapping(values.as_ptr(), ptr, N);
            ptr::copy_nonoverlapping(&*value as *const T, ptr.add(N), 1);
            result.assume_init()
        };

        ConstArr(values)
    }

    pub const fn concat<const M: usize>(self, other: ConstArr<T, M>) -> ConstArr<T, { N + M }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let other = ManuallyDrop::new(other);
        let left = unsafe { ptr::read(&this.0) };
        let right = unsafe { ptr::read(&other.0) };
        let left = ManuallyDrop::new(left);
        let right = ManuallyDrop::new(right);

        let mut result = MaybeUninit::<[T; N + M]>::uninit();
        let ptr = result.as_mut_ptr() as *mut T;

        let values = unsafe {
            ptr::copy_nonoverlapping(left.as_ptr(), ptr, N);
            ptr::copy_nonoverlapping(right.as_ptr(), ptr.add(N), M);
            result.assume_init()
        };

        ConstArr(values)
    }

    pub const fn concat_append<const M: usize>(
        self,
        other: ConstArr<T, M>,
        value: T,
    ) -> ConstArr<T, { N + M + 1 }> {
        self.concat(other).append(value)
    }
}
