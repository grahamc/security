use notmuch_sys::{notmuch_messages_t, notmuch_messages_valid,
                  notmuch_messages_get, notmuch_messages_move_to_next,
                  notmuch_messages_destroy,
                  TRUE};
use std::marker::PhantomData;
use nixvulns::NMMessage;
use nixvulns::NMQuery::NMQuery;
use std::sync::Arc;
use nixvulns::memhelp::{mktrace_trace_static, logtrace};


#[derive(Debug)]
pub struct NMMessages<'a> {
    handle: *mut notmuch_messages_t,
    phantom: PhantomData<&'a notmuch_messages_t>,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

pub fn new<'a>(handle: *mut notmuch_messages_t, trace: &Option<String>, query: Arc<NMQuery>) -> NMMessages<'a> {
    NMMessages {
        handle: handle,
        _trace: mktrace_trace_static(trace, "Messages"),
        phantom: PhantomData,
        query: query,
    }
}

impl<'a> Iterator for NMMessages<'a> {
    type Item = NMMessage::NMMessage<'a>;

    fn next(&mut self) -> Option<NMMessage::NMMessage<'a>> {
        unsafe {
            if notmuch_messages_valid(self.handle) == TRUE {
                let cur = notmuch_messages_get(self.handle);
                if ! cur.is_null() {
                    notmuch_messages_move_to_next(self.handle);
                    return Some(NMMessage::new(cur, &self._trace, self.query.clone()));
                }
            }
        }

        return None;
    }
}

impl<'a> Drop for NMMessages<'a> {
    fn drop(&mut self) {
        logtrace("Dropping Messages", &self._trace);
        unsafe {
            notmuch_messages_destroy(self.handle);
        }
    }
}
