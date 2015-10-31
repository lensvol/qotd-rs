use std::io::Cursor;
use std::io::BufReader;
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

impl StrfileHeader {
    pub fn flag_is_set(&self, mask: Flags) -> bool {
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

    pub fn new(filename: String) -> Result<StrfileHeader, Error> {
        let mut header = StrfileHeader {
            version: 1,
            number_of_strings: 0,
            longest_length: 0,
            shortest_length: 0,
            flags: 0,
            delim: 0,
            offsets: vec![],
        };
	    let mut header_field = [0u8; 21];

        let handle = try!(File::open(filename.clone()));
        let mut file = BufReader::new(&handle);
        try!(file.read(&mut header_field));
	    let mut buf = Cursor::new(&header_field[..]);

	    header.version = buf.read_u32::<BigEndian>().unwrap();
	    header.number_of_strings = buf.read_u32::<BigEndian>().unwrap();
	    header.longest_length = buf.read_u32::<BigEndian>().unwrap();
	    header.shortest_length = buf.read_u32::<BigEndian>().unwrap();
	    header.flags = buf.read_u32::<BigEndian>().unwrap();
	    header.delim = header_field[20];

        try!(file.seek(SeekFrom::Current(3)));
        for _ in 1 .. header.number_of_strings + 1 {
            let mut raw_offset = [0u8; 4];
            try!(file.read(&mut raw_offset));
            let mut buf = Cursor::new(&raw_offset[..]);
            let offset = buf.read_u32::<BigEndian>().unwrap();
            header.offsets.push(offset);
        }

        let header = header;
        Ok(header)
    }

}

