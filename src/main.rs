extern crate rand;
extern crate clap;
extern crate byteorder;

extern crate qotd_rs;

use clap::App;
use rand::Rng;

use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Error;

use std::fs::File;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

use qotd_rs::strfile::StrfileHeader;

fn rot13(c: char) -> char {
    let base = match c {
        'a'...'z' => 'a' as u8,
        'A'...'Z' => 'A' as u8,
        _ => return c
    };

    let rotated = ((c as u8) - base + 13) % 26;
    (rotated + base) as char
}

#[test]
fn rot13_test() {
    assert!("Hello, world!".to_owned() == "Uryyb, jbeyq!".to_owned().chars().map(rot13).collect::<String>());
}

fn display_strfile_header(header: &StrfileHeader) {
    println!("Version:\t{}", header.version);
	println!("Strings:\t{}", header.number_of_strings);
	println!("Longest:\t{}", header.longest_length);
	println!("Shortest:\t{}", header.shortest_length);
	println!("Delimeter:\t{:?}", header.delim as char);

	println!("Randomized:\t{}", header.is_random());
	println!("Ordered:\t{}", header.is_ordered());
	println!("ROT13:\t\t{}", header.is_rotated());
	println!("Comments:\t{}\n", header.has_comments());
}

fn read_quote_from_file(reader: &mut BufReader<File>, delim: &u8) -> String {
    let mut quote = String::new();
    let mut buffer = String::new();
    let mut found = false;

    let bytes  = vec![*delim, 10];
    let separator = String::from_utf8(bytes).unwrap();

    while !found {
        reader.read_line(&mut buffer).unwrap();
        if buffer.len() > 0 && buffer != separator {
            quote.push_str(&buffer);
            buffer.clear();
        } else {
            found = true;
        }
    };

    quote
}


fn load_indexed_quotes(filename: String, header: &StrfileHeader) -> Result<Vec<String>, Error>{
    let mut quotes = Vec::new();
    let file = try!(File::open(filename));
    let mut reader = BufReader::new(file);

    for offset in &header.offsets {
        try!(reader.seek(SeekFrom::Start(*offset as u64)));
        let quote = read_quote_from_file(&mut reader, &header.delim);
        if header.is_rotated() {
            quotes.push(quote.chars().map(rot13).collect::<String>());
        } else {
            quotes.push(quote);
        }
    }
    Ok(quotes)
}


fn load_raw_quotes(filename: String) -> Result<Vec<String>, Error> {
	let mut quotes = Vec::new();

	let f = try!(File::open(filename));
    let file = BufReader::new(&f);
    let mut quote = "".to_string();
    for line in file.lines() {
    	let l = line.unwrap();
    	if l == "%" {
    		quotes.push(quote);
    		quote = "".to_string();
    	} else {
    		quote.push_str(&l);
    		quote.push_str(&"\n");
    	}
    }

    Ok(quotes)
}

fn tcp_handler(bind_addr: String, quotes: &Vec<String>) {
	match TcpListener::bind(bind_addr.trim()) {
        Ok(listener) => {
	        for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let ref quote = choose_random_one(quotes);
                stream.write(&quote.as_bytes()).unwrap();
            }
        },
        Err(e) => println!("{:?}", e)
    }

}

fn udp_handler(bind_addr: String, quotes: &Vec<String>) {
	match  UdpSocket::bind(bind_addr.trim()) {
        Ok(socket) => {
	        loop {
		        let mut buf = [0; 10];
		        let (_, src) = socket.recv_from(&mut buf).unwrap();

		        let ref quote = choose_random_one(quotes);
		        socket.send_to(&quote.as_bytes(), &src).unwrap();
	        }
        },
        Err(e) => println!("{:?}", e),
    }
}

fn choose_random_one(quotes: &Vec<String>) -> &String {
    let random_index = rand::thread_rng().gen_range(0, quotes.len());
    &quotes[random_index]
}

fn main() {
    let matches = App::new("qotd-rs")
						.version("0.1")
						.author("Kirill Borisov <borisov.kir@gmail.com>")
	                    .args_from_usage(
							"-b --bind=[ADDR] 'Bind at specified address.'
							<FILENAME> 'Sets quotes file to use.'")
						.get_matches();

	let bind_addr_str = matches.value_of("ADDR").unwrap_or("127.0.0.1:17").to_string();
	let quotes_fn = matches.value_of("FILENAME").unwrap().to_string();

	let header = StrfileHeader::new(quotes_fn.clone() + ".dat");
    let quotes = match header {
        Ok(h) => {
            display_strfile_header(&h);
            load_indexed_quotes(quotes_fn.clone(), &h).unwrap()
        },
        Err(_) => load_raw_quotes(quotes_fn).unwrap()
    };

	let shared_quotes = Arc::new(quotes);

	let udp_bind_addr = bind_addr_str.clone();
	let tcp_bind_addr = bind_addr_str.clone();
	let tcp_quotes = shared_quotes.clone();
	let udp_quotes = shared_quotes.clone();

	println!("TCP/UDP server listening on port {}.", bind_addr_str);

	let tcp_listener_handle = thread::spawn(move || {
		tcp_handler(udp_bind_addr, &tcp_quotes);
	});

	let udp_listener_handle = thread::spawn(move || {
		udp_handler(tcp_bind_addr, &udp_quotes);
	});

	tcp_listener_handle.join().unwrap();
	udp_listener_handle.join().unwrap();
}
