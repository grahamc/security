use std::ffi::CString;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub fn str_to_cstr(my_str: &str) -> CString {
    let os_str = OsString::from(my_str);
    let bytes = os_str.as_bytes();
    CString::new(bytes).unwrap()
}

pub fn logtrace(whatup: &str, trace: &Option<String>) {
    if let &Some(ref trace) = trace {
        let mut trace = trace.clone();

        if trace.len() > 250 {
            let mut start_at = 250;
            while !trace.is_char_boundary(start_at) {
                start_at += 1;
            }
            trace.truncate(start_at);
            trace.push_str("TRUNCATED");
        }
        //println!("~ {} >>>{}<<<", whatup, trace);
    } else {
        //println!("~ {} MISSING TRACE", whatup);
    }
}

pub fn mktrace_trace_static(prefix: &Option<String>, suffix: &str) -> Option<String> {
    if let &Some(ref prefix) = prefix {
        Some(format!("{}.{}", prefix, suffix))
    } else {
        Some(format!("MISSING.{}", suffix))
    }
}

pub fn mktrace_trace_id(prefix: &Option<String>, suffix: String) -> Option<String> {
    if let &Some(ref prefix) = prefix {
        Some(format!("{}.{}", prefix, suffix))
    } else {
        Some(format!("MISSING.{}", suffix))
    }
}
