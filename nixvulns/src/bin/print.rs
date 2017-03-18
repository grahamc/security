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
    handle: *mut notmuch_sys::notmuch_database_t
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
            handle: db
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
                db: self.0.clone()
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
                    query: Arc::new(query)
                })
            } else {
                Err(status)
            }
        }
    }
}

impl Drop for NMDB {
    fn drop(&mut self) {
        unsafe {
            notmuch_database_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMQuery {
    handle:  *mut notmuch_sys::notmuch_query_t,
    db: Arc<NMDB>
}

impl NMQuery {

}

impl Drop for NMQuery {
    fn drop(&mut self) {
        unsafe {
            notmuch_query_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMThreads {
    handle: *mut notmuch_sys::notmuch_threads_t,
    query: Arc<NMQuery>
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
                    return Some(NMThread{
                        handle: cur,
                        query: self.query.clone()
                    });
                }
            }
        }

        return None;
    }
}

impl Drop for NMThreads {
    fn drop(&mut self) {
        unsafe {
            notmuch_threads_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMThread {
    handle: *mut notmuch_sys::notmuch_thread_t,
    query: Arc<NMQuery>
}

impl NMThread {
    fn thread_id(&self) -> String {
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_thread_id(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    fn subject(&self) -> String {
        unsafe {
            CStr::from_ptr(
                notmuch_thread_get_subject(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    fn tags(&self) -> NMTags {
        unsafe {
            NMTags {
                handle: notmuch_thread_get_tags(self.handle)
            }
        }
    }

    fn messages(&self) -> NMMessages {
        unsafe {
            NMMessages {
                handle: notmuch_thread_get_messages(self.handle)
            }
        }
    }
}

impl Drop for NMThread {
    fn drop(&mut self) {
        unsafe {
            notmuch_thread_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMTags {
    handle: *mut notmuch_sys::notmuch_tags_t,
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
        unsafe {
            notmuch_tags_destroy(self.handle);
        }
    }
}


#[derive(Debug)]
struct NMMessages {
    handle: *mut notmuch_sys::notmuch_messages_t,
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
                    notmuch_messages_move_to_next(self.handle);
                    return Some(NMMessage {
                        handle: cur
                    });
                }
            }
        }

        return None;
    }
}

impl Drop for NMMessages {
    fn drop(&mut self) {
        unsafe {
            notmuch_messages_destroy(self.handle);
        }
    }
}


#[derive(Debug)]
struct NMMessage {
    handle: *mut notmuch_sys::notmuch_message_t,
}

impl NMMessage {
    fn message_id(&self) -> String {
        unsafe {
            CStr::from_ptr(
                notmuch_message_get_message_id(self.handle)
            ).to_str().unwrap().to_string()
        }
    }

    fn header(&self, header: &str) -> String {
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
        unsafe {
            CStr::from_ptr(
                notmuch_message_get_filename(self.handle)
            ).to_str().unwrap().to_string()
        }
    }
}

impl Drop for NMMessage {
    fn drop(&mut self) {
        unsafe {
            notmuch_message_destroy(self.handle);
        }
    }
}



fn main() {
    let mut nm = NMDBArc::open("/home/grahamc/.mail/grahamc");
    let mut threads = nm.search_threads("tag:needs-triage and tag:nixossec date:2017-02-22..").unwrap();

    let mut by_suggested_package: HashMap<String,Vec<Arc<NMThread>>> = HashMap::new();
    for mut thread in threads {
        let thread = Arc::new(thread);

        let mut tags: Vec<String> = vec![];
        for tag in thread.tags() {
            let mut splits = tag.splitn(2, ":");
            match (splits.nth(0)) {
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


            for message in thread.messages() {
                println!("### {}, `{}`",
                         message.header("from"),
                         message.message_id());
                println!("<!-- {} -->\n", message.filename());

                let mut mailtxt: String = String::new();
                let mut msg = File::open(message.filename()).unwrap();
                msg.read_to_string(&mut mailtxt);
                let parsed = mailparse::parse_mail(mailtxt.as_bytes()).unwrap();

                println!("\n```\n{}\n```\n", parsed.get_body().unwrap());

                if parsed.subparts.len() > 0 {
                    println!("Additional Parts");
                    for part in parsed.subparts {
                        println!("<details><summary>Additional Parts</summary>");
                        println!("\n```\n{}\n```\n", part.get_body().unwrap());
                        println!("</details>");
                    }
                    println!("\n---\n");
                }

            }
            println!("</details>");

        }
        println!("");
    }
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
