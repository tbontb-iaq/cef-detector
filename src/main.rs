use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::metadata,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

struct App {
    path: String,
    size: u128,
    cef_files: HashSet<String>,
}

const CEF_FILES: [&str; 5] = [
    r"chrome_100_percent\.pak",
    r"chrome_crashpad_handler",
    r"chrome-sandbox",
    r"libcef\.so",
    r"resources\.pak",
];

fn locate(regex: &str) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();

    let child = Command::new("locate")
        .args(["-r", regex])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = BufReader::new(child.stdout.unwrap());

    for line in reader.lines() {
        let path = line.unwrap();
        let metadata = metadata(path.as_str()).unwrap();
        if metadata.is_file() {
            let path_buf = PathBuf::from(path.as_str());
            let parent = path_buf.parent().unwrap().to_str().unwrap();

            map.entry(String::from(parent))
                .or_insert(HashSet::new())
                .insert(path);
        }
    }

    map
}

fn disk_usage(folder: &str) -> u128 {
    let output = Command::new("du").args(["-s", folder]).output().unwrap();
    let output = String::from_utf8_lossy(&output.stdout);
    let size_str = output.split_ascii_whitespace().next().unwrap();
    let size = size_str.parse::<u128>().unwrap();
    size
}

fn format_size(size: u128) -> String {
    let units = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.2}{}", size, units[unit_index])
}

fn main() {
    println!("locating...");

    let mut map = BTreeMap::<String, App>::new();

    for file in CEF_FILES {
        let locate_result = locate(&format!(r"\/{}$", file));
        for (path, set) in locate_result {
            let size = disk_usage(path.as_str());
            map.entry(path.clone())
                .or_insert(App {
                    size,
                    path,
                    cef_files: HashSet::new(),
                })
                .cef_files
                .extend(set);
        }
    }

    let mut total_size = 0u128;

    for app in map.values() {
        println!("软件目录：{}", app.path);
        println!("软件大小：{}", format_size(app.size));
        for path in &app.cef_files {
            println!("\t{}", path);
        }
        total_size += app.size;
        println!();
    }

    println!(
        "此电脑中共有 {} 个 Chromium 内核应用 ({})",
        map.len(),
        format_size(total_size)
    );
}
