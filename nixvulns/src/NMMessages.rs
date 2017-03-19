use notmuch_sys::{notmuch_messages_t, notmuch_messages_valid,
                  notmuch_messages_get, notmuch_messages_move_to_next,
                  notmuch_messages_destroy,
                  TRUE};
use nixvulns::NMMessage;
use nixvulns::NMQuery::NMQuery;
use std::sync::Arc;
use nixvulns::memhelp::{mktrace_trace_static, logtrace};


#[derive(Debug)]
pub struct NMMessages {
    handle: *mut notmuch_messages_t,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

pub fn new(handle: *mut notmuch_messages_t, query: Arc<NMQuery>, trace: &Option<String>) -> NMMessages {
    NMMessages {
        handle: handle,
        query: query,
        _trace: mktrace_trace_static(trace, "Messages"),
    }
}

impl Iterator for NMMessages {
    type Item = Arc<NMMessage::NMMessage>;

    fn next(&mut self) -> Option<Arc<NMMessage::NMMessage>> {
        unsafe {
            if notmuch_messages_valid(self.handle) == TRUE {
                let cur = notmuch_messages_get(self.handle);
                if ! cur.is_null() {
                    notmuch_messages_move_to_next(self.handle);

                    {
                        let checkdb = self.query.messages.read().unwrap();
                        if checkdb.contains_key(&cur) {
                            return Some(checkdb.get(&cur).unwrap().clone());
                        }
                    }

                    let mut writedb = self.query.messages.write().unwrap();
                    if writedb.contains_key(&cur) {
                        return Some(writedb.get(&cur).unwrap().clone());
                    } else {
                        let msg = Arc::new(NMMessage::new(cur, &self._trace));
                        writedb.insert(cur, msg);
                        return Some(writedb.get(&cur).unwrap().clone());
                    }
                }
            }
        }

        return None;
    }
}

impl Drop for NMMessages {
    fn drop(&mut self) {
        logtrace("Dropping Messages", &self._trace);
        unsafe {
            notmuch_messages_destroy(self.handle);
        }
    }
}
