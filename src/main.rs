use std::env;
use std::fs;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::path;
use encoding_rs_io::DecodeReaderBytes;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("usage: spin2ascii path_containing_spin_files");
    }
    let pathname = &args[1];
    println!("Searching {} for UTF-16 .spin files to convert...", pathname);

    // Iterate over all the files in the selected folder to find the .spin source files.
    // Open each of these and convert to ASCII if they are currently UTF-16.
    let entries = fs::read_dir(pathname);
    if entries.is_err() {
        println!("error: Failed to iterate over entries in {} - {}", pathname, entries.unwrap_err());
        return;
    }
    for entry in entries.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = path.extension();
        if path.is_file() && extension.is_some() && extension.unwrap() == "spin" {
            process_spin_file(&path);
        }
    }
}

fn process_spin_file(path: &path::Path) {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let bytes = fs::read(path);
    if bytes.is_err() {
        println!("error: Failed to open and read contents of {} - {}", filename, bytes.unwrap_err());
        return;
    }
    let bytes = bytes.unwrap();
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xFE {
        // Doesn't start with a UTF-16 BOM so ignore it.
        return;
    }

    // This is a UTF-16 file so massage it into an ASCII file.
    println!("{}", filename);
    let decoder = DecodeReaderBytes::new(&bytes[..]);
    let reader = BufReader::new(decoder);
    let mut new_path = path.to_path_buf();
    new_path.set_extension("spin_new");
    let new_file = fs::File::create(&new_path);
    if new_file.is_err() {
        println!("error: Failed to create {} - {}", new_path.to_str().unwrap(), new_file.unwrap_err());
        return;
    }
    let mut new_file = BufWriter::new(new_file.unwrap());
    for line in reader.lines() {
        if line.is_err() {
            println!("error: Failed to read line of text from {}", filename);
        }
        let line = line.unwrap();
        for ch in line.chars() {
            if ch.len_utf8() == 1 {
                write!(&mut new_file, "{}", ch).unwrap();
            }
            else {
                match ch {
                    '\u{2500}' => write!(&mut new_file, "-").unwrap(),
                    '\u{2502}' => write!(&mut new_file, "|").unwrap(),
                    '\u{251c}'| '\u{2524}' | '\u{2514}' | '\u{2534}' | '\u{253c}' |
                    '\u{2518}' | '\u{252c}' | '\u{2510}' | '\u{250c}'  => write!(&mut new_file, "+").unwrap(),
                    '\u{2022}' => write!(&mut new_file, "*").unwrap(),
                    '\u{b1}' => write!(&mut new_file, "+/-").unwrap(),
                    _ => panic!("{}-{:X}", ch, ch as u32)
                }
            }
        }
        write!(&mut new_file, "\r\n").unwrap();
    }
}