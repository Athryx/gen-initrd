use clap::clap_app;

use std::fs::File;
use std::io::{self, Read, Write};
use std::process::exit;

const MAGIC: u64 = 0x39f298aa4b92e836;
const ALIGN: u64 = 8;

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
enum EntryType {
	Any = 0,
	EarlyInit = 1,
	PartList = 2,
	FsSever = 3,
	AhciServer = 4,
}

#[repr(C)]
#[derive(Debug)]
struct Header {
	magic: u64,
	len: u64,
}

impl Header {
	fn new(len: u64) -> Self {
		Header {
			magic: MAGIC,
			len,
		}
	}

	fn as_bytes(&self) -> &[u8] {
		unsafe {
			let ptr = self as *const _ as *const u8;
			std::slice::from_raw_parts(ptr, std::mem::size_of::<Self> ())
		}
	}
}

#[derive(Debug)]
struct Entry<'a> {
	typ: EntryType,
	name: &'a str,
	data: Vec<u8>,
}

impl Entry<'_> {
	fn new(typ: EntryType, path: &str) -> io::Result<Entry> {
		let mut file = File::open(path)?;
		let mut data = Vec::new();
		file.read_to_end(&mut data)?;

		Ok(Entry {
			typ,
			name: path,
			data,
		})
	}

	fn name_bytes(&self) -> &[u8] {
		self.name.as_bytes()
	}

	fn data_bytes(&self) -> &[u8] {
		&self.data[..]
	}

	// does not set name and data offset
	fn as_raw(&self) -> EntryRaw {
		EntryRaw {
			typ: self.typ as u64,
			name: 0,
			name_len: self.name.len() as u64,
			data: 0,
			data_len: self.data.len() as u64,
		}
	}
}

#[repr(C)]
#[derive(Debug)]
struct EntryRaw {
	typ: u64,
	name: u64,
	name_len: u64,
	data: u64,
	data_len: u64,
}

impl EntryRaw {
	fn as_bytes(&self) -> &[u8] {
		unsafe {
			let ptr = self as *const _ as *const u8;
			std::slice::from_raw_parts(ptr, std::mem::size_of::<Self> ())
		}
	}
}

fn align_up(n: u64, align: u64) -> u64 {
	(n + align - 1) & !(align - 1)
}

fn align_to(vec: &mut Vec<u8>, align: u64) {
	let len = vec.len() as u64;
	let aligned_len = align_up(len, align);

	for _ in 0..(aligned_len - len) {
		vec.push(0);
	}
}

fn to_initrd(entries: &Vec<Entry>) -> Vec<u8> {
	// current offset of data in file
	let mut offset = (std::mem::size_of::<Header> () + std::mem::size_of::<EntryRaw> () * entries.len ()) as u64;

	let mut out = Vec::new();

	let header = Header::new(entries.len() as u64);
	out.extend_from_slice(header.as_bytes());

	for entry in entries.iter() {
		let mut raw_entry = entry.as_raw();

		raw_entry.name = offset;
		offset += align_up(raw_entry.name_len, ALIGN);

		raw_entry.data = offset;
		offset += align_up(raw_entry.data_len, ALIGN);

		out.extend_from_slice(raw_entry.as_bytes());
	}

	for entry in entries.iter() {
		align_to(&mut out, ALIGN);
		out.extend_from_slice(entry.name_bytes());

		align_to(&mut out, ALIGN);
		out.extend_from_slice(entry.data_bytes());
	}

	out
}

fn main() {
	let matches = clap_app!(("gen-initrd") =>
		(version: "0.1.0")
		(about: "Simple utility to generate initrd image for the aurora kernel")
		(@arg ("early-init"): -i --init <EXECUTABLE> "First executable spawned by kernel which is responsible for mounting the root filesytem and spawning the init process")
		(@arg ("part-list"): -p --("part-list") <FILE> "File read by early-init which describes which filesytem drivers to use for which partitions and where to mount them")
		(@arg ("fs-server"): -f --fs <EXECUTABLE> "Filesystem server which filesytem drivers will connect to")
		(@arg ("ahci-server"): -a --ahci <EXECUTABLE> "Ahci server to allow filesytem drivers to communicate with drives")
		(@arg out: -o <FILE> "Output file to save initrd to")
		(@arg files: [FILE] ... "additional files to include in initrd")
	).get_matches();

	let early_init = matches.value_of("early-init").unwrap();
	let part_list = matches.value_of("part-list").unwrap();
	let fs_server = matches.value_of("fs-server").unwrap();
	let ahci_server = matches.value_of("ahci-server").unwrap();
	let other_files = matches.values_of("files");

	let mk_entry = |typ, path| {
		match Entry::new(typ, path) {
			Ok(entry) => entry,
			Err(err) => {
				eprintln!("Could not read from file {}: {}", path, err);
				exit(1);
			},
		}
	};

	let mut entries = vec![
		mk_entry(EntryType::EarlyInit, early_init),
		mk_entry(EntryType::PartList, part_list),
		mk_entry(EntryType::FsSever, fs_server),
		mk_entry(EntryType::AhciServer, ahci_server),
	];

	if let Some(files) = other_files {
		for file in files {
			entries.push(mk_entry(EntryType::Any, file));
		}
	}

	let out_path = matches.value_of("out").unwrap();
	let mut out_file = match File::create(out_path)
	{
		Ok(file) => file,
		Err(_) => {
			eprintln!("Could not create output file {}", out_path);
			exit(1);
		}
	};

	let initrd_vec = to_initrd(&entries);
	if let Err(_) = out_file.write_all(&initrd_vec[..])
	{
		eprintln!("Could not write initrd to output file {}", out_path);
		exit(1);
	}
}
