extern crate notmuch_sys;
extern crate mailparse;

//use mime_multipart;
use std::fs::File;
use std::ffi::{CStr, CString};
use notmuch_sys::*;
use std::ptr;

use std::collections::HashMap;
use std::sync::Arc;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::io::{Read, Write};

fn logtrace(whatup: &str, trace: &Option<String>) {
    if let &Some(ref trace) = trace {
        let mut trace = trace.clone();

        if trace.len() > 250 {
            let mut startAt = 250;
            while !trace.is_char_boundary(startAt) {
                startAt += 1;
            }
            trace.truncate(startAt);
            trace.push_str("TRUNCATED");
        }

        println!("~ {} >>>{}<<<", whatup, trace);
    } else {
        println!("~ {} MISSING TRACE", whatup);
    }
}

fn mktrace_trace_static(prefix: &Option<String>, suffix: &str) -> Option<String> {
    if let &Some(ref prefix) = prefix {
        Some(format!("{}.{}", prefix, suffix))
    } else {
        Some(format!("MISSING.{}", suffix))
    }
}

fn mktrace_trace_id(prefix: &Option<String>, suffix: String) -> Option<String> {
    if let &Some(ref prefix) = prefix {
        Some(format!("{}.{}", prefix, suffix))
    } else {
        Some(format!("MISSING.{}", suffix))
    }
}

fn str_to_cstr(my_str: &str) -> std::ffi::CString {
    /*
    println!("{:?}", b"From\0");
    let mut foo = "From".as_bytes();
    let mut foo = CString::new(foo);
    println!("{:?}", foo);
    panic!("lol");
     */
    let os_str = OsString::from(my_str);
    let bytes = os_str.as_bytes();
    CString::new(bytes).unwrap()
}


/*fn str_to_i8(my_str: &str) -> *const i8 {
    CString::new(
        OsString::from(my_str).as_bytes()
    ).unwrap().as_ptr() as *const u8 as *const i8
}

//fn str_to_i8(input: *const i8) -> String {
//    return input as *const u8;
    //CString::new(
    //    OsString::from(my_str).as_bytes()
    //).unwrap().as_ptr() as *const u8 as *const i8
//}
*/

#[derive(Debug)]
struct NMDB {
    handle: *mut notmuch_sys::notmuch_database_t,
    _trace: Option<String>
}

struct NMDBArc (Arc<NMDB>);

impl NMDBArc {
    fn open(path: &str) -> NMDBArc {
        let mut db = ptr::null_mut();
        let cpath = str_to_cstr(path);
        let cptr = cpath.as_ptr();

        unsafe {
            notmuch_database_open(
                cptr,
                notmuch_database_mode_t::READ_ONLY,
                &mut db
            );
        }

        return NMDBArc(Arc::new(NMDB{
            handle: db,
            _trace: Some("db".to_string()),
        }));
    }

    fn search(&self, query: &str) -> NMQuery {
        let cquery = str_to_cstr(query);
        let cptr = cquery.as_ptr();
        unsafe {
            NMQuery {
                handle: notmuch_query_create(
                    self.0.handle,
                    cptr
                ),
                db: self.0.clone(),
                _trace: mktrace_trace_static(&(self.0)._trace, "query"),
            }
        }
    }

    fn search_threads(&self, query: &str) -> Result<NMThreads,notmuch_status_t> {
        let query = self.search(query);

        let mut threads = ptr::null_mut();

        unsafe {
            let status = notmuch_query_search_threads_st(query.handle, &mut threads);

            if status == notmuch_status_t::SUCCESS {
                Ok(NMThreads {
                    handle: threads,
                    query: Arc::new(query),
                    _trace: mktrace_trace_static(&(self.0)._trace, "search_threads"),
                })
            } else {
                Err(status)
            }
        }
    }
}

impl Drop for NMDB {
    fn drop(&mut self) {
        logtrace("Dropping DB", &self._trace);
        unsafe {
            notmuch_database_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMQuery {
    handle:  *mut notmuch_sys::notmuch_query_t,
    db: Arc<NMDB>,
    _trace: Option<String>
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

#[derive(Debug)]
struct NMThreads {
    handle: *mut notmuch_sys::notmuch_threads_t,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

impl NMThreads {
}

impl Iterator for NMThreads {
    type Item = NMThread;

    fn next(&mut self) -> Option<NMThread> {
        unsafe {
            if notmuch_threads_valid(self.handle) == notmuch_sys::TRUE {
                let cur = notmuch_threads_get(self.handle);

                if ! cur.is_null() {
                    notmuch_threads_move_to_next(self.handle);
                    let mut thread = NMThread{
                        handle: cur,
                        query: self.query.clone(),
                        _trace: mktrace_trace_static(&self._trace, "unitit"),
                    };
                    thread._trace = mktrace_trace_id(&self._trace, thread.thread_id());
                    logtrace("Updated thread's trace", &thread._trace);
                    return Some(thread);
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

#[derive(Debug)]
struct NMThread {
    handle: *mut notmuch_sys::notmuch_thread_t,
    query: Arc<NMQuery>,
    _trace: Option<String>,
}

impl NMThread {
    fn thread_id(&self) -> String {
        logtrace("Fetch thread id", &self._trace);
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_thread_id(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    fn subject(&self) -> String {
        logtrace("Fetch subject", &self._trace);
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_subject(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    fn tags(&self) -> NMTags {
        logtrace("Fetch tags", &self._trace);
        unsafe {
            NMTags {
                handle: notmuch_thread_get_tags(self.handle),
                _trace: mktrace_trace_static(&self._trace, "Tags")
            }
        }
    }

    fn messages(&self) -> NMMessages {
        logtrace("Fetch Messages", &self._trace);
        unsafe {
            NMMessages {
                handle: notmuch_thread_get_messages(self.handle),
                _trace: mktrace_trace_static(&self._trace, "Messages")
            }
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

#[derive(Debug)]
struct NMTags {
    handle: *mut notmuch_sys::notmuch_tags_t,
    _trace: Option<String>,
}

impl NMTags {
}

impl Iterator for NMTags {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        unsafe {
            if notmuch_tags_valid(self.handle) == notmuch_sys::TRUE {
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


#[derive(Debug)]
struct NMMessages {
    handle: *mut notmuch_sys::notmuch_messages_t,
    _trace: Option<String>,
}

impl NMMessages {
}

impl Iterator for NMMessages {
    type Item = NMMessage;

    fn next(&mut self) -> Option<NMMessage> {
        unsafe {
            if notmuch_messages_valid(self.handle) == notmuch_sys::TRUE {
                let cur = notmuch_messages_get(self.handle);
                if ! cur.is_null() {
                    let mut msg = NMMessage {
                        handle: cur,
                        _trace: mktrace_trace_static(&self._trace, "unknownmsg"),
                    };
                    logtrace("Initializing trace", &msg._trace);
                    let mid = msg.message_id();
                    let midlen = mid.len();
                    msg._trace = mktrace_trace_id(&self._trace, mid);
                    if midlen > 150 {
                        logtrace("Fuckery is afoot", &msg._trace);
                    }




                    notmuch_messages_move_to_next(self.handle);
                    return Some(msg);
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


#[derive(Debug)]
struct NMMessage {
    handle: *mut notmuch_sys::notmuch_message_t,
    _trace: Option<String>,
}

impl NMMessage {
    fn message_id(&self) -> String {
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

    fn header(&self, header: &str) -> String {
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

    fn filename(&self) -> String {
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
            // notmuch_message_destroy(self.handle);
        }
    }
}



fn main() {
    let mut messages: Vec<NMMessage> = vec![];

    {
        let mut nm = NMDBArc::open("/home/grahamc/.mail/grahamc");
        let mut threads = nm.search_threads("tag:needs-triage and tag:nixossec date:2017-02-22..").unwrap();

        for mut thread in threads {
            for message in thread.messages() {
                messages.push(message);
            }
        }
    }

    for msg in messages {
        println!("{}", msg.message_id());
    }







    /*
    let mut by_suggested_package: HashMap<String,Vec<Arc<NMThread>>> = HashMap::new();

    let mut nm = NMDBArc::open("/home/grahamc/.mail/grahamc");
    let mut threads = nm.search_threads("tag:needs-triage and tag:nixossec date:2017-02-22..").unwrap();


    for mut thread in threads {
        let thread = Arc::new(thread);

        let mut tags: Vec<String> = vec![];
        for tag in thread.tags() {
            let mut splits = tag.splitn(2, ":");
            match splits.nth(0) {
                Some("suggested") => {
                    if let Some(suggestion) = splits.next() {
                        by_suggested_package.entry(suggestion.to_string()).or_insert(vec!()).push(thread.clone());
                    } else {
                        println!("{:?}", splits);
                    }
                }
                fallback => {
                    // println!("{:?}", fallback);
                }
            }
        }
    };

    for (tag, threads) in by_suggested_package {
        println!("## {}", tag);

        for thread in threads {
            println!("<details>");
            println!("<summary><strong>{}</strong></summary>\n", thread.subject());

            println!("{:?} <-> {:?}", thread.messages().next(), thread.messages().next());

            for message in thread.messages() {
                println!("<!-- next: {} -->\n", message.filename());
                println!("### {}, `{}`",
                         message.header("from"),
                         message.message_id());

                let mut mailtxt: String = String::new();
                let mut msg = File::open(message.filename()).unwrap();
                msg.read_to_string(&mut mailtxt);
                let parsed = mailparse::parse_mail(mailtxt.as_bytes()).unwrap();

                // println!("\n```\n{}\n```\n", parsed.get_body().unwrap());

                if parsed.subparts.len() > 0 {
                    println!("Additional Parts");
                    for part in parsed.subparts {
                        println!("<details><summary>Additional Parts</summary>");
                        // println!("\n```\n{}\n```\n", part.get_body().unwrap());
                        println!("</details>");
                    }
                    println!("\n---\n");
                }

            }
            println!("</details>");

        }
        println!("");
    }

    */
/*

    let mut threads2 = nm.search_threads("tag:unread and date:2017-02-22..").unwrap();
    println!("threads");
    println!("{:?}", threads2);
    for mut thread in threads2 {
        // println!("thread:{:?}", thread.thread_id());

        for tag in thread.tags() {
            // tags.push(tag);
        }
    };

    /*while let Some(thread) = threads.next_thread() {

        break;
    }*/

println!("bye");*/
}
