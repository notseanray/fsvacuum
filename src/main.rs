use dirs::cache_dir;
use dirs::home_dir;
use ignore::DirEntry;
use ignore::WalkBuilder;
use ignore::WalkState::*;
use lazy_static::lazy_static;
use num_cpus::get;
use std::env;
use std::ffi::OsStr;
use std::fs::read_dir;

use std::fs::remove_dir_all;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

lazy_static! {
    static ref ARGS: Vec<String> = env::args().skip(1).collect();
    static ref TYPES: Vec<&'static str> = parse_types(ARGS.to_vec());
}

macro_rules! remove_type {
    ($cache_dir: expr, $dir:expr, $name:expr) => {
        if remove_dir_all($cache_dir.join($dir)).is_ok() {
            println!("removed {} cache", $name);
        }
    };
}

fn clear_cache() {
    if !ARGS.contains(&"clean".to_owned()) {
        return;
    }
    let cache = cache_dir();
    let cache_dir = match cache {
        Some(v) => v,
        None => home_dir().unwrap().join(".cache"),
    };
    if !cache_dir.exists() {
        panic!("could not find cache directory!");
    }
    remove_type!(cache_dir, "go-build", "go build");
    remove_type!(cache_dir, "pylint", "pylint");
    remove_type!(cache_dir, "pylint", "pylint");
    remove_type!(cache_dir, "typescript", "typescript");
    remove_type!(cache_dir, "yarn", "yarn");
    remove_type!(cache_dir, "chromium", "chromium");
    remove_type!(cache_dir, "pip", "pip");
    remove_type!(cache_dir, "mozilla", "firefox");
    remove_type!(cache_dir, "expo", "RN expo");
    remove_type!(cache_dir, "JetBrains", "JetBrains");
    remove_type!(cache_dir, "rollup-plugin-rust", "rollup rust");
    remove_type!(cache_dir, "nim", "nim");
    remove_type!(cache_dir, "eas-cli", "eas");
    remove_type!(cache_dir, "deno", "deno");
    remove_type!(cache_dir, "esbuild", "esbuild");
}

fn nvim_swap() {
    let read_dir = match read_dir(home_dir().unwrap().join(".local/share/nvim/swap")) {
        Ok(v) => v,
        Err(_) => return,
    };
    for file in read_dir {
        let path = file.unwrap().path();
        if !ARGS.contains(&"clean".to_owned()) {
            println!("found {} in nvim swap", path.to_string_lossy());
            return;
        }
        if remove_file(&path).is_ok() {
            if let Some(name) = path.file_name() {
                println!("removed {} from nvim swap", name.to_string_lossy());
            }
        }
    }
}

fn v_modules() {
    let v_path =  home_dir().unwrap().join(".vmodules");
    if !ARGS.contains(&"clean".to_owned()) {
        println!("found v modules");
        return;
    }
    if ARGS.contains(&"clean".to_string()) {
        remove_dir_all(&v_path).unwrap_or(());
        println!("erased {}", v_path.to_string_lossy());
    }
}

fn parse_types(args: Vec<String>) -> Vec<&'static str> {
    let mut types = Vec::new();
    for arg in args {
        match arg.as_str() {
            "rust" => types.push("target"),
            "js" => types.push("node_modules"),
            "zig" => {
                types.push("zig-out");
                types.push("zig-cache");
            }
            "v" => v_modules(),
            "nvim" => nvim_swap(),
            "cache" => clear_cache(),
            _ => {}
        }
    }
    if types.is_empty() {
        println!("zero types detected, exiting");
        std::process::exit(0);
    }
    types
}

fn handle_path(path: &Path, folder_name: &OsStr) {
    let name = folder_name.to_string_lossy().to_string();
    if !TYPES.contains(&name.as_str()) {
        return;
    }
    let nested: Vec<String> = path
        .display()
        .to_string()
        .split('/')
        .map(|x| x.to_string())
        .collect();
    let mut count = 0;
    nested.iter().for_each(|x| {
        if TYPES.contains(&x.as_str()) {
            count += 1;
        }
    });
    if count > 1 {
        return;
    }
    if ARGS.contains(&"clean".to_string()) {
        remove_dir_all(path).unwrap_or(());
        println!("erased {}", path.to_string_lossy());
        return;
    }
    println!("found path {}", path.to_string_lossy());
}

fn main() {
    let startup = Instant::now();
    let (tx, rx) = crossbeam_channel::bounded::<DirEntry>(100);
    let stdout_thread = thread::spawn(move || {
        for dent in rx {
            if let Some(v) = dent.file_type() {
                if v.is_file() {
                    continue;
                }
            }
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
        "done in {} ms using {core_count} threads",
        startup.elapsed().as_millis()
    );
}
