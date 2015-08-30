extern crate rand;

use rand::Rng;

use std::env;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;


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

fn choose_random_one(quotes: Vec<String>) -> String {
    let random_index = rand::thread_rng().gen_range(0, quotes.len());
    quotes[random_index].clone()
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() == 1 {
		println!("File with quotes is not specified!");
	} else {
		let loaded_quotes = load_quotes(args[1].clone());
		println!("{}", choose_random_one(loaded_quotes));
	}
}
