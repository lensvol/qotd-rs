extern crate rand;
extern crate clap;
extern crate byteorder;

use clap::App;
use rand::Rng;

use std::io::Cursor;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;

use std::fs::File;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

use byteorder::{BigEndian, ReadBytesExt};


fn print_strfile_header(handle: File) {
	let mut header_field = [0u8; 21];

    let mut file = BufReader::new(&handle);
    file.read(&mut header_field).unwrap();
	let mut buf = Cursor::new(&header_field[..]);

	let version: u32 = buf.read_u32::<BigEndian>().unwrap();
	let number_of_strings: u32 = buf.read_u32::<BigEndian>().unwrap();
	let longest_length = buf.read_u32::<BigEndian>().unwrap();
	let shortest_length = buf.read_u32::<BigEndian>().unwrap();
	let flags = buf.read_u32::<BigEndian>().unwrap();
	let delim = header_field[20];

	println!("Version:\t{}", version);
	println!("Strings:\t{}", number_of_strings);
	println!("Longest:\t{}", longest_length);
	println!("Shortest:\t{}", shortest_length);
	println!("Delimeter:\t{:?}", delim as char);

	let flag_set = |mask| {
		if flags & mask == 1 { "yes" } else { "no" }
	};

	println!("Randomized:\t{}", flag_set(0x1));	
	println!("Ordered:\t{}", flag_set(0x2));
	println!("Encrypted:\t{}", flag_set(0x4));	
	println!("Comments:\t{}\n", flag_set(0x8));
}


fn load_raw_quotes(filename: String) -> Vec<String> {
	let mut quotes = Vec::new();

	let f = match File::open(filename) {
		Ok(file) => file,
		Err(e) => {
			panic!("{}", e);
		}
	};
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

    quotes
}

fn tcp_handler(bind_addr: String, quotes: &Vec<String>) {
	let listener = TcpListener::bind(bind_addr.trim()).unwrap();

	for stream in listener.incoming() {
		let mut stream = stream.unwrap();
		let ref quote = choose_random_one(quotes);
		stream.write(&quote.as_bytes()).unwrap();
	}
}

fn udp_handler(bind_addr: String, quotes: &Vec<String>) {
	let socket = UdpSocket::bind(bind_addr.trim()).unwrap();
	loop {
		let mut buf = [0; 10];
		let (_, src) = socket.recv_from(&mut buf).unwrap();

		let ref quote = choose_random_one(quotes);
		socket.send_to(&quote.as_bytes(), &src).unwrap();
	};	
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

	match File::open(quotes_fn.clone() + ".dat") {
		Ok(file) => print_strfile_header(file),
		Err(e) => {println!("{:?}", e);}
	};

	let shared_quotes = Arc::new(load_raw_quotes(quotes_fn));

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
