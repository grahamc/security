use notmuch_sys::{notmuch_threads_valid, notmuch_threads_get,
                  notmuch_threads_move_to_next,
                  notmuch_threads_destroy, notmuch_threads_t,
                  notmuch_query_search_threads_st,
                  notmuch_status_t, TRUE};
use nixvulns::NMQuery::NMQuery;
use nixvulns::NMThread;
use nixvulns::memhelp::{mktrace_trace_static, logtrace};
use std::sync::Arc;
use std::ptr;

#[derive(Debug)]
pub struct NMThreads {
    handle: *mut notmuch_threads_t,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

pub fn new(db_trace: &Option<String>, query: NMQuery) -> Result<NMThreads, notmuch_status_t> {
    let mut threads = ptr::null_mut();

    unsafe {
        let status = notmuch_query_search_threads_st(query.handle, &mut threads);

        if status == notmuch_status_t::SUCCESS {
            Ok(NMThreads {
                handle: threads,
                query: Arc::new(query),
                _trace: mktrace_trace_static(db_trace, "search_threads"),
            })
        } else {
            Err(status)
        }
    }
}

impl NMThreads {
}

impl Iterator for NMThreads {
    type Item = Arc<NMThread::NMThread>;

    fn next(&mut self) -> Option<Arc<NMThread::NMThread>> {
        unsafe {
            if notmuch_threads_valid(self.handle) == TRUE {
                let cur = notmuch_threads_get(self.handle);

                if ! cur.is_null() {
                    notmuch_threads_move_to_next(self.handle);

                    {
                        let checkdb = self.query.threads.read().unwrap();
                        if checkdb.contains_key(&cur) {
                            return Some(checkdb.get(&cur).unwrap().clone());
                        }
                    }

                    let mut writedb = self.query.threads.write().unwrap();
                    if writedb.contains_key(&cur) {
                        return Some(writedb.get(&cur).unwrap().clone());
                    } else {
                        let msg = Arc::new(NMThread::new(cur, self.query.clone(), &self._trace));
                        writedb.insert(cur, msg);
                        return Some(writedb.get(&cur).unwrap().clone());
                    }
                }
            }
        }

        return None;
    }
}

impl Drop for NMThreads {
    fn drop(&mut self) {
        logtrace("Dropping Threads", &self._trace);
        unsafe {
            notmuch_threads_destroy(self.handle);
        }
    }
}
