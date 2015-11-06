use std::io::Cursor;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Error;

use std::fs::File;

use byteorder::{BigEndian, ReadBytesExt};

pub struct StrfileHeader {
    pub version: u32,
    pub number_of_strings: u32,
    pub longest_length: u32,
    pub shortest_length: u32,
    pub flags: u32,
    pub delim: u8,
    pub offsets: Vec<u32>,
}

pub enum Flags {
    Random = 0x1,
    Ordered = 0x2,
    Rotated = 0x4,
    HasComments = 0x8
}

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
    let original_str = "Hello, world!".to_owned();
    let encrypted_str = "Uryyb, jbeyq!".to_owned();
    assert!(original_str == encrypted_str.chars().map(rot13).collect::<String>());
}

fn read_quote_from_file(reader: &mut BufReader<File>, delim: &u8) -> String    {
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

impl StrfileHeader {
    fn flag_is_set(&self, mask: Flags) -> bool {
        self.flags & (mask as u32) == 1
    }

    pub fn is_random(&self) -> bool {
        self.flag_is_set(Flags::Random)
    }

    pub fn is_rotated(&self) -> bool {
        self.flag_is_set(Flags::Rotated)
    }

    pub fn is_ordered(&self) -> bool {
        self.flag_is_set(Flags::Ordered)
    }

    pub fn has_comments(&self) -> bool {
        self.flag_is_set(Flags::HasComments)
    }

    pub fn read_quotes(&self, filename: String) -> Result<Vec<String>, Error>{
        let mut quotes = Vec::new();
        let file = try!(File::open(filename));
        let mut reader = BufReader::new(file);

        for offset in &self.offsets {
            try!(reader.seek(SeekFrom::Start(*offset as u64)));
            let quote = read_quote_from_file(&mut reader, &self.delim);
            if self.is_rotated() {
                quotes.push(quote.chars().map(rot13).collect::<String>());
            } else {
                quotes.push(quote);
            }
        }
        Ok(quotes)
    }

    pub fn parse(filename: String) -> Result<StrfileHeader, Error>  {
	    let mut header_field = [0u8; 21];

        let handle = try!(File::open(filename.clone()));
        let mut file = BufReader::new(&handle);
        try!(file.read(&mut header_field));
        let mut buf = Cursor::new(&header_field[..]);

        let version = buf.read_u32::<BigEndian>().unwrap();
        let number_of_strings = buf.read_u32::<BigEndian>().unwrap();
        let longest_length = buf.read_u32::<BigEndian>().unwrap();
        let shortest_length = buf.read_u32::<BigEndian>().unwrap();
        let flags = buf.read_u32::<BigEndian>().unwrap();
        let delim = header_field[20];
        let mut offsets = Vec::new();

        try!(file.seek(SeekFrom::Current(3)));
        for _ in 1 .. number_of_strings + 1 {
            let mut raw_offset = [0u8; 4];
            try!(file.read(&mut raw_offset));
            let mut buf = Cursor::new(&raw_offset[..]);
            let offset = buf.read_u32::<BigEndian>().unwrap();
            offsets.push(offset);
        }

        let header =  StrfileHeader {
            version: version,
            number_of_strings: number_of_strings,
            longest_length: longest_length,
            shortest_length: shortest_length,
            flags: flags,
            delim: delim,
            offsets: offsets,
        };
        Ok(header)
    }
}

