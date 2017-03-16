extern crate notmuch_sys;

use std::ffi::{CStr, CString};
use notmuch_sys::*;
use std::ptr;

use std::sync::Arc;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::io::{Read, Write};


fn str_to_cstr(my_str: &str) -> std::ffi::CString {
    CString::new(
        OsString::from(my_str).as_bytes()
    ).unwrap()
}

fn str_to_i8(my_str: &str) -> *const i8 {
    CString::new(
        OsString::from(my_str).as_bytes()
    ).unwrap().as_ptr() as *const u8 as *const i8
}

#[derive(Debug)]
struct NMDB {
    handle: *mut notmuch_sys::notmuch_database_t
}

struct NMDBArc (Arc<NMDB>);

impl NMDBArc {
    fn open(path: &str) -> NMDBArc {
        let mut db = ptr::null_mut();

        unsafe {
            notmuch_database_open(
                str_to_cstr(path).as_ptr(),
                notmuch_database_mode_t::READ_ONLY,
                &mut db
            );
        }

        return NMDBArc(Arc::new(NMDB{
            handle: db
        }));
    }

    fn search(&mut self, query: &str) -> NMQuery {
        unsafe {
            NMQuery {
                handle: notmuch_query_create(
                    Arc::get_mut(&mut self.0).unwrap().handle,
                    str_to_i8(query)
                ),
                db: self.0.clone()
            }
        }
    }

    fn search_threads(&mut self, query: &str) -> Result<NMThreads,notmuch_status_t> {
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

}

impl Drop for NMThread {
    fn drop(&mut self) {
        unsafe {
            notmuch_thread_destroy(self.handle);
        }
    }
}



fn main() {
    println!("hi");

    let mut nm = NMDBArc::open("/home/grahamc/.mail/grahamc");
    println!("nm");
    let mut threads = nm.search_threads("tag:needs-triage and date:2017-02-22..").unwrap();
    println!("threads");
    println!("{:?}", threads);
    for thread in threads {
        println!("{:?}", thread);
    };

    /*while let Some(thread) = threads.next_thread() {

        break;
    }*/

println!("bye");
}
