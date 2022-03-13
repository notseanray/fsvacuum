use ignore::DirEntry;
use ignore::WalkBuilder;
use ignore::WalkState::*;
use lazy_static::lazy_static;
use num_cpus::get;
use std::env;
use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

lazy_static! {
    static ref ARGS: Vec<String> = env::args().skip(1).collect();
    static ref TYPES: Vec<String> = parse_types(ARGS.to_vec());
}

fn parse_types(args: Vec<String>) -> Vec<String> {
    let mut types = Vec::new();
    for arg in args {
        match arg.as_str() {
            "rust" => types.push("target".to_string()),
            "js" => types.push("node_modules".to_string()),
            _ => {}
        }
    }
    if types.len() == 0 {
        println!("zero types detected, exiting");
        std::process::exit(0);
    }
    types
}

fn handle_path(path: &Path, folder_name: &OsStr) {
    let name = folder_name.to_string_lossy().to_string();
    if !TYPES.contains(&name) {
        return;
    }
    let nested: Vec<String> = path
        .display()
        .to_string()
        .split("/")
        .map(|x| x.to_string())
        .collect();
    let mut count = 0;
    nested.iter().for_each(|x| {
        if !TYPES.contains(x) {
            return;
        }
        count += 1;
    });
    if count > 1 {
        return;
    }
    if ARGS.contains(&"clean".to_string()) {
        remove_dir_all(path).unwrap_or_else(|_| return);
        println!("erased {:#?}", path);
        return;
    }
    println!("found path {:#?}", path);
}

fn main() {
    let startup = Instant::now();
    let (tx, rx) = crossbeam_channel::bounded::<DirEntry>(100);
    let stdout_thread = thread::spawn(move || {
        for dent in rx {
            match dent.file_type() {
                Some(v) => {
                    if v.is_file() {
                        continue;
                    }
                }
                None => {}
            };
            let current_path = PathBuf::from(dent.path());
            if current_path.exists() {
                handle_path(dent.path(), dent.file_name());
            }
        }
    });
    let core_count = get();
    let walker = WalkBuilder::new("./")
        .threads(core_count)
        .hidden(false)
        .git_ignore(false)
        .build_parallel();
    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(v) = result {
                let _ = tx.send(v);
            }
            Continue
        })
    });
    drop(tx);
    stdout_thread.join().unwrap();
    println!(
        "done in {:#?} using {core_count} threads",
        startup.elapsed()
    );
}
