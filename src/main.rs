extern crate rand;
extern crate clap;

use clap::App;
use rand::Rng;

use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;

use std::fs::File;
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;


fn load_quotes(filename: String) -> Vec<String>{
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

	let shared_quotes = Arc::new(load_quotes(quotes_fn));

	let udp_bind_addr = bind_addr_str.clone();
	let tcp_bind_addr = bind_addr_str.clone();
	let tcp_quotes = shared_quotes.clone();
	let udp_quotes = shared_quotes.clone();

	println!("TCP server listening on port {}.", tcp_bind_addr);
	println!("UDP server listening on port {}.", udp_bind_addr);

	let tcp_listener_handle = thread::spawn(move || {
		tcp_handler(udp_bind_addr, &tcp_quotes);
	});

	let udp_listener_handle = thread::spawn(move || {
		udp_handler(tcp_bind_addr.clone(), &udp_quotes);
	});

	tcp_listener_handle.join().unwrap();
	udp_listener_handle.join().unwrap();
}
