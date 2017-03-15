extern crate notmuch_sys;

use std::ffi::{CStr, CString};
use notmuch_sys::*;
use std::ptr;

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

impl NMDB {
    fn open(path: &str) -> NMDB {
        let mut db = ptr::null_mut();

        unsafe {
            notmuch_database_open(
                str_to_cstr(path).as_ptr(),
                notmuch_database_mode_t::READ_ONLY,
                &mut db
            );
        }

        return NMDB{
            handle: db
        };
    }

    fn search(&mut self, query: &str) -> NMQuery {
        unsafe {
            NMQuery {
                handle: notmuch_query_create(
                    self.handle,
                    str_to_i8(query)
                ),
                db: self
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
                    query: query
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
struct NMQuery<'a> {
    handle:  *mut notmuch_sys::notmuch_query_t,
    db: &'a NMDB

}

impl<'a> NMQuery<'a> {

}

impl<'a> Drop for NMQuery<'a> {
    fn drop(&mut self) {
        unsafe {
            notmuch_query_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMThreads<'a> {
    handle: *mut notmuch_sys::notmuch_threads_t,
    query: NMQuery<'a>
}

impl<'a> NMThreads<'a> {
}

impl<'a> Iterator for NMThreads<'a> {
    type Item = NMThread<'a>;

    fn next(&'a mut self) -> Option<NMThread<'a>> {
        unsafe {
            if notmuch_threads_valid(self.handle) == notmuch_sys::TRUE {
                let cur = notmuch_threads_get(self.handle);

                if ! cur.is_null() {
                    notmuch_threads_move_to_next(self.handle);
                    return Some(NMThread{
                        handle: cur,
                        query: &self.query
                    });
                }
            }
        }

        return None;
    }
}

impl<'a> Drop for NMThreads<'a> {
    fn drop(&mut self) {
        unsafe {
            notmuch_threads_destroy(self.handle);
        }
    }
}

#[derive(Debug)]
struct NMThread<'a> {
    handle: *mut notmuch_sys::notmuch_thread_t,
    query: &'a NMQuery<'a>
}

impl<'a> NMThread<'a> {

}

impl<'a> Drop for NMThread<'a> {
    fn drop(&mut self) {
        unsafe {
            notmuch_thread_destroy(self.handle);
        }
    }
}



fn main() {
    println!("hi");

    let mut nm = NMDB::open("/home/grahamc/.mail/grahamc");
    println!("nm");
    let mut threads = nm.search_threads("tag:needs-triage and date:2017-02-22..").unwrap();
    println!("threads");
    println!("{:?}", threads);
    /*while let Some(thread) = threads.next_thread() {
        println!("{:?}", thread);
        break;
    }*/

println!("bye");
}
