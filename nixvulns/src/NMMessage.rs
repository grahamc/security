use notmuch_sys::{notmuch_message_t, notmuch_message_get_message_id,
                  notmuch_message_get_header,
                  notmuch_message_get_filename, notmuch_message_destroy};
use nixvulns::memhelp::{logtrace, str_to_cstr, mktrace_trace_id,
                        mktrace_trace_static};
use std::ffi::CStr;
use std::sync::Arc;
use nixvulns::NMQuery::NMQuery;

#[derive(Debug)]
pub struct NMMessage {
    handle: *mut notmuch_message_t,
    _trace: Option<String>,
}

pub fn new (cur: *mut notmuch_message_t, trace: &Option<String>) -> NMMessage {
    let mut msg = NMMessage {
        handle: cur,
        _trace: mktrace_trace_static(trace, "unknownmsg"),
    };
    logtrace("Initializing trace", &msg._trace);
    let mid = msg.message_id();
    let midlen = mid.len();
    msg._trace = mktrace_trace_id(trace, mid);
    if midlen > 150 {
        logtrace("Fuckery is afoot", &msg._trace);
    }

    return msg;
}

impl NMMessage {
    pub fn message_id(&self) -> String {
        logtrace("Getting message_id", &self._trace);
        unsafe {
            let mid = notmuch_message_get_message_id(self.handle);
            if mid.is_null() {
                logtrace("message_id is null", &self._trace);
                panic!("message_id Should not be null?")
            } else {
                let cstr = CStr::from_ptr(mid);
                let asstr = cstr.to_str();
                let unwrapped = asstr.unwrap().to_string();
                return unwrapped;
            }
        }
    }

    pub fn header(&self, header: &str) -> String {
        logtrace("Fetching header", &self._trace);
        let h = str_to_cstr(header);
        let hptr = h.as_ptr();
        unsafe {
            CStr::from_ptr(
                notmuch_message_get_header(self.handle,
                                          hptr
                )
            ).to_str().unwrap().to_string()
        }
    }

    pub fn filename(&self) -> String {
        logtrace("Fetching filename", &self._trace);
        unsafe {
            let filename_ptr = notmuch_message_get_filename(self.handle);
            assert!(!filename_ptr.is_null());

            let cstr = CStr::from_ptr(filename_ptr);
            let asstr = cstr.to_str();
            let unwrapped = asstr.unwrap().to_string();
            return unwrapped;
        }
    }
}

impl Drop for NMMessage {
    fn drop(&mut self) {
        logtrace("Dropping message", &self._trace);
        unsafe {
            notmuch_message_destroy(self.handle);
        }
    }
}
