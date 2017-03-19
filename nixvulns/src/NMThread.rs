use std::sync::Arc;
use notmuch_sys::{notmuch_thread_get_thread_id,
                  notmuch_thread_get_subject, notmuch_thread_get_tags,
                  notmuch_thread_get_messages, notmuch_thread_destroy,
                  notmuch_thread_t};
use nixvulns::NMQuery::NMQuery;
use nixvulns::NMTags;
use nixvulns::NMMessages;
use nixvulns::memhelp::{mktrace_trace_id, mktrace_trace_static,
                        logtrace};
use std::ffi::CStr;

#[derive(Debug)]
pub struct NMThread {
    handle: *mut notmuch_thread_t,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

pub fn new (cur: *mut notmuch_thread_t, query: Arc<NMQuery>, trace: &Option<String>) -> NMThread {
    let mut thread = NMThread {
        handle: cur,
        query: query,
        _trace: mktrace_trace_static(trace, "unitit"),
    };
    thread._trace = mktrace_trace_id(trace, thread.thread_id());
    logtrace("Updated thread's trace", &thread._trace);
    return thread;
}

impl NMThread {
    pub fn thread_id(&self) -> String {
        logtrace("Fetch thread id", &self._trace);
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_thread_id(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    pub fn subject(&self) -> String {
        logtrace("Fetch subject", &self._trace);
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_subject(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    pub fn tags(&self) -> NMTags::NMTags {
        logtrace("Fetch tags", &self._trace);
        unsafe {
            NMTags::new(notmuch_thread_get_tags(self.handle), &self._trace)
        }
    }

    pub fn messages(&self) -> NMMessages::NMMessages {
        logtrace("Fetch Messages", &self._trace);
        unsafe {
            NMMessages::new(
                notmuch_thread_get_messages(self.handle),
                self.query.clone(),
                &self._trace
            )
        }
    }
}

impl Drop for NMThread {
    fn drop(&mut self) {
        logtrace("Dropping thread", &self._trace);
        unsafe {
            notmuch_thread_destroy(self.handle);
        }
    }
}
