#![no_std]

use core::ffi::c_void;
use core::mem::{align_of, size_of, MaybeUninit};

#[cfg(test)]
mod tests;

/// Allocates `[u8;size]` memory on stack and invokes `closure` with this slice as argument.
///
///
/// # Potential segfaults or UB
///
/// When using this function in wrong way your program might get UB or segfault "for free":
/// - Using memory allocated by `with_alloca` outside of it e.g closure is already returned but you somehow
/// managed to store pointer to memory and use it.
/// - Allocating more memory than thread stack size.
///
///
///     This will trigger segfault on stack overflow.
///
///
///
#[inline]
pub fn with_raw<R, F>(size: usize, f: F) -> R
where
    F: FnOnce(*mut u8) -> R,
{
    type Callback = unsafe extern "C" fn(ptr: *mut u8, data: *mut c_void);
    extern "C" {
        fn c_with_alloca(size: usize, cb: Callback, data: *mut c_void);
    }

    let mut f = Some(f);
    let mut ret = None::<R>;

    let ref mut f = |ptr: *mut u8| {
        ret = Some(f.take().unwrap()(ptr));
    };

    #[inline(always)]
    fn with_fn_of_val<F>(_: &mut F) -> Callback
    where
        F: FnMut(*mut u8),
    {
        unsafe extern "C" fn trampoline<F: FnMut(*mut u8)>(ptr: *mut u8, data: *mut c_void) {
            (&mut *data.cast::<F>())(ptr);
        }

        trampoline::<F>
    }

    // SAFETY: The function `c_with_alloca` will always return a valid pointer.
    unsafe {
        c_with_alloca(size, with_fn_of_val(f), <*mut _>::cast::<c_void>(f));
    }

    ret.unwrap()
}

#[inline]
pub fn with_bytes<R, F>(size: usize, f: F) -> R
where
    F: FnOnce(&mut [MaybeUninit<u8>]) -> R,
{
    crate::with_raw(size, |ptr| {
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr as *mut MaybeUninit<u8>, size) };
        f(slice)
    })
}

#[inline]
pub fn with_bytes_zeroed<R, F>(size: usize, f: F) -> R
where
    F: FnOnce(&mut [u8]) -> R,
{
    crate::with_bytes(size, |slice| {
        let slice = unsafe { &mut *(slice as *mut [MaybeUninit<u8>] as *mut [u8]) };
        slice.fill(0);
        f(slice)
    })
}

#[inline]
pub fn with_slice<T, R, F>(count: usize, f: F) -> R
where
    F: FnOnce(&mut [MaybeUninit<T>]) -> R,
{
    let size = count * size_of::<T>() + align_of::<T>() - 1;
    with_raw(size, |memory| {
        let mut raw_memory = memory as *mut MaybeUninit<T>;

        // ensure each element is properly aligned
        let offset = raw_memory as usize % align_of::<T>();
        if offset != 0 {
            raw_memory = unsafe { raw_memory.add(align_of::<T>() - offset) };
        }

        let slice = unsafe { core::slice::from_raw_parts_mut(raw_memory, count) };

        f(slice)
    })
}

#[inline]
pub fn with_slice_zeroed<T, R, F>(count: usize, f: F) -> R
where
    F: FnOnce(&mut [MaybeUninit<T>]) -> R,
{
    with_slice(count, |slice| {
        unsafe {
            core::ptr::write_bytes(slice.as_mut_ptr(), 0, count);
        }

        f(slice)
    })
}

#[inline]
pub unsafe fn with_slice_assume_init<T, R, F>(count: usize, f: F) -> R
where
    F: FnOnce(&mut [T]) -> R,
{
    with_slice(count, |slice| {
        let slice = unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) };
        f(slice)
    })
}

#[inline]
pub unsafe fn with_slice_zeroed_assume_init<T, R, F>(count: usize, f: F) -> R
where
    F: FnOnce(&mut [T]) -> R,
{
    with_slice_zeroed(count, |slice| {
        let slice = unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) };
        f(slice)
    })
}

/// Allocates `T` on stack space.
#[inline]
pub fn with<T, R, F>(f: F) -> R
where
    F: FnOnce(&mut MaybeUninit<T>) -> R,
{
    with_slice(1, |slice| {
        f(unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut MaybeUninit<T>) })
    })
}

#[inline]
pub fn with_zeroed<T, R, F>(f: F) -> R
where
    F: FnOnce(&mut MaybeUninit<T>) -> R,
{
    with(|data| {
        unsafe {
            core::ptr::write_bytes(data.as_mut_ptr(), 0, 1);
        }

        f(data)
    })
}

#[inline]
pub unsafe fn with_assume_init<T, R, F>(f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    with(|data| f(unsafe { &mut *(data as *mut MaybeUninit<T> as *mut T) }))
}

#[inline]
pub unsafe fn with_zeroed_assume_init<T, R, F>(f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    with_zeroed(|data| f(unsafe { &mut *(data as *mut MaybeUninit<T> as *mut T) }))
}
