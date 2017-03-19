use notmuch_sys::{notmuch_tags_t, notmuch_tags_valid,
                  notmuch_tags_get, notmuch_tags_move_to_next,
                  notmuch_tags_destroy, TRUE};
use std::ffi::CStr;
use nixvulns::memhelp::{logtrace, mktrace_trace_static};

#[derive(Debug)]
pub struct NMTags {
    handle: *mut notmuch_tags_t,
    _trace: Option<String>,
}

pub fn new(handle: *mut notmuch_tags_t, trace: &Option<String>) -> NMTags {
    NMTags {
        handle: handle,
        _trace: mktrace_trace_static(trace, "Tags")
    }
}

impl NMTags {
}

impl Iterator for NMTags {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        unsafe {
            if notmuch_tags_valid(self.handle) == TRUE {
                let cur = notmuch_tags_get(self.handle);
                if ! cur.is_null() {
                    notmuch_tags_move_to_next(self.handle);
                    return Some(
                        CStr::from_ptr(cur).to_str().unwrap().to_string()
                    );
                }
            }
        }

        return None;
    }
}

impl Drop for NMTags {
    fn drop(&mut self) {
        logtrace("Dropping Tags", &self._trace);
        unsafe {
            notmuch_tags_destroy(self.handle);
        }
    }
}
