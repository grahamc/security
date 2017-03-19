use notmuch_sys;
use nixvulns::memhelp;
use nixvulns::{NMQuery,NMThreads};


use std::sync::Arc;
use std::ptr;

#[derive(Debug)]
pub struct NMDB {
    handle: *mut notmuch_sys::notmuch_database_t,
    _trace: Option<String>
}

impl Drop for NMDB {
    fn drop(&mut self) {
        memhelp::logtrace("Dropping DB", &self._trace);
        unsafe {
            notmuch_sys::notmuch_database_destroy(self.handle);
        }
    }
}


pub struct NMDBArc (Arc<NMDB>);

impl NMDBArc {
    pub fn open(path: &str) -> NMDBArc {
        let mut db = ptr::null_mut();
        let cpath = memhelp::str_to_cstr(path);
        let cptr = cpath.as_ptr();

        unsafe {
            notmuch_sys::notmuch_database_open(
                cptr,
                notmuch_sys::notmuch_database_mode_t::READ_ONLY,
                &mut db
            );
        }

        return NMDBArc(Arc::new(NMDB{
            handle: db,
            _trace: Some("db".to_string()),
        }));
    }

    fn search(&self, query: &str) -> NMQuery::NMQuery {
        NMQuery::new(self.0.handle, self.0.clone(), &(self.0)._trace,
                     query)
    }

    pub fn search_threads(&self, query: &str) -> Result<NMThreads::NMThreads,notmuch_sys::notmuch_status_t> {
        NMThreads::new(
            &(self.0)._trace,
            self.search(query)
        )
    }
}
