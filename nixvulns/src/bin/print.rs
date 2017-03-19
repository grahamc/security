extern crate nixvulns;

extern crate notmuch_sys;
extern crate mailparse;

//use mime_multipart;
use std::fs::File;
use nixvulns::NMDB::NMDBArc;
use nixvulns::NMThread::NMThread;
use std::collections::HashMap;
use std::sync::Arc;
use std::io::Read;


fn main() {
    let mut by_suggested_package: HashMap<String,Vec<Arc<NMThread>>> = HashMap::new();

    let nm = NMDBArc::open("/home/grahamc/.mail/grahamc");
    let threads = nm.search_threads("tag:needs-triage and date:2017-02-22..").unwrap();


    for thread in threads {
        let thread = Arc::new(thread);

        for tag in thread.tags() {
            let mut splits = tag.splitn(2, ":");
            match splits.nth(0) {
                Some("suggested") => {
                    if let Some(suggestion) = splits.next() {
                        by_suggested_package.entry(suggestion.to_string()).or_insert(vec!()).push(thread.clone());
                    } else {
                        println!("WHAT? {:?}", splits);
                    }
                }
                fallback => {}
            }
        }
    };

    for (tag, threads) in by_suggested_package {
        println!("## {}", tag);

        for thread in threads {
            println!("<details>");
            println!("<summary><strong>{}</strong></summary>\n", thread.subject());

            for message in thread.messages() {
                println!("<!-- next: {} -->\n", message.filename());
                println!("##### {}, `{}`",
                         message.header("from"),
                         message.message_id());

                let mut mailtxt: String = String::new();
                let mut msg = File::open(message.filename()).unwrap();
                msg.read_to_string(&mut mailtxt);
                let parsed = mailparse::parse_mail(mailtxt.as_bytes()).unwrap();

                if parsed.subparts.len() == 0 {
                    // print_body(&parsed);
                } else {
                    for part in parsed.subparts {
                        println!("<details><summary>Additional Parts</summary>");
                        // print_body(&parsed.subparts.first().unwrap());
                        println!("</details>");
                    }
                    println!("\n---\n");

                    break;
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
