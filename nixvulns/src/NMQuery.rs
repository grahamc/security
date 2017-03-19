use std::sync::Arc;
use notmuch_sys::{notmuch_query_destroy, notmuch_query_create,
                  notmuch_query_t, notmuch_database_t,
                  notmuch_message_t, notmuch_thread_t};
use nixvulns::memhelp::{str_to_cstr, logtrace, mktrace_trace_static};
use nixvulns::NMDB::NMDB;
use nixvulns::NMMessage::NMMessage;
use nixvulns::NMThread::NMThread;
use std::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug)]
pub struct NMQuery {
    pub handle:  *mut notmuch_query_t,
    db: Arc<NMDB>,
    pub messages: RwLock<HashMap<*mut notmuch_message_t,Arc<NMMessage>>>,
    pub threads: RwLock<HashMap<*mut notmuch_thread_t,Arc<NMThread>>>,
    _trace: Option<String>
}

pub fn new(db_handle: *mut notmuch_database_t, db: Arc<NMDB>, db_trace: &Option<String>, query: &str) -> NMQuery {
    let cquery = str_to_cstr(query);
    let cptr = cquery.as_ptr();
    unsafe {
        NMQuery {
            handle: notmuch_query_create(
                db_handle,
                cptr
            ),
            db: db,
            messages: RwLock::new(HashMap::new()),
            threads: RwLock::new(HashMap::new()),
            _trace: mktrace_trace_static(db_trace, "query"),
        }
    }
}

impl NMQuery {
}

impl Drop for NMQuery {
    fn drop(&mut self) {
        logtrace("Dropping query", &self._trace);
        unsafe {
            notmuch_query_destroy(self.handle);
        }
    }
}
