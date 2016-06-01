use std::ffi;
use jack_sys as j;

/// Collects strings from an array of c-strings into a Rust vector of strings
/// and frees the memory pointed to by `ptr`. The end of the array is marked by
/// the value of the c-string being the null pointer. `ptr` may be `null`, in
/// which case nothing (deallocating) is done and an empty vector is returned.
pub unsafe fn collect_strs(ptr: *const *const i8) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    };
    let len = {
        let mut len = 0;
        while !ptr.offset(len).is_null() {
            len += 1;
        };
        len
    };
    let mut strs = Vec::with_capacity(len as usize);
    for i in 0..len {
        let cstr_ptr = *ptr.offset(i);
        let s = ffi::CStr::from_ptr(cstr_ptr)
            .to_string_lossy()
            .into_owned();
        strs.push(s);
    }
    j::jack_free(ptr as *mut ::libc::c_void);
    strs
}
