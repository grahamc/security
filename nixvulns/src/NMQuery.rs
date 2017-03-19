use std::sync::Arc;
use notmuch_sys::{notmuch_query_destroy, notmuch_query_create,
                  notmuch_query_t, notmuch_database_t};
use nixvulns::memhelp::{str_to_cstr, logtrace, mktrace_trace_static};
use nixvulns::NMDB::NMDB;

#[derive(Debug)]
pub struct NMQuery {
    pub handle:  *mut notmuch_query_t,
    db: Arc<NMDB>,
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
