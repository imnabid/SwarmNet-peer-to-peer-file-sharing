use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::{fs, net::IpAddr};

pub fn process_search(datagram: &str, ip_addr: IpAddr) {
    let mut datagram = datagram.lines().map(|l| l);
    let req_type = &datagram.next().unwrap()[5..];
    let mut search_result: HashMap<String, String> = HashMap::new();

    match req_type {
        "search_query" => {
            let target_string = &datagram.next().unwrap()[6..];
            println!("got a search request data:{}", target_string);
            let length = search(target_string, &mut search_result);
            println!("total files matched: {}", length);
            if length > 0 {
                //send the metadata of available files to the requesting peer
                let socket_addr = SocketAddr::new(ip_addr, 7878);
                //set timeout of 10secs
                let timeout = Duration::from_secs(10);
                // let mut stream = TcpStream::connect(socket_addr)

                let mut stream = TcpStream::connect_timeout(&socket_addr, timeout)
                    .expect("failed to send search results");
                let metadata = format!("type:hash\nno_of_files:{}\n",length);
                stream.write(metadata.as_bytes()).unwrap();
                for  data in search_result.values_mut() {
                    let msg = data.as_bytes();
                    stream
                    .write(msg)
                    .expect("failed to send metadata for search query");
                }
            }
        }
        _ => println!("not a search query"),
    }
}

fn search(target_string: &str, search_result: &mut HashMap<String, String>) -> usize {
    //to remove "\0" i.e null character as datagram is 1024 bytes
    let target_string = target_string.trim_end_matches('\0');
    let target_string = target_string.trim();
    let subtarget_string: Vec<&str> = target_string.split(' ').collect();

    let dir_path = "./file_hash";
    let files = fs::read_dir(dir_path).unwrap();

    for path in files {
        let entry = path.unwrap();
        let path = entry.path();

        let contents = fs::read_to_string(path).unwrap();
        let mut lines = contents.lines();
        // let _content_type = &lines.next().unwrap()[5..];

        let title = &lines.next().unwrap()[10..];
        let file_hash = &lines.next().unwrap()[10..];

        for substring in subtarget_string.iter() {
            let regex_pattern = format!(r"(?i)\b(?:\w*{}\w*)\b", substring);
            let regex = Regex::new(&regex_pattern).expect("Failed to compile regex");

            if regex.is_match(&title) {
                if !search_result.contains_key(file_hash) {
                    search_result.insert(file_hash.to_string(), contents);
                    break;
                }
            }
        }
    }

    search_result.len()
}