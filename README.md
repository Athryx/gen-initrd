# gen-initrd

A simple utility to generate an init ram disk (initrd) for the aurora kernel

The initrd contains several different programs:

- early-init: mounts the root filesystem and starts the init process
- part-list: specifies which partitions have which filesystems and where to mount them
- fs-server: filesystem server that filesystem drivers talk to
- ahci-server: ahci driver that filesystem drivers use to communicate with drives
- one or more filesystem drivers

## format

the format is very simple, and consists of one header and many entries

### header layout

	struct InitrdHeader {
		magic: u64,
		len: u64,
	}

the header is always at the very beginning of the initrd

magic will always be initialized to 0x39f298aa4b92e836

len spicifies how many entries there are

### entry layout

	struct InitrdEntry {
		type: u64,
		name: u64,
		name_len: u64,
		data: u64,
		data_len: u64,
	}

the entry list starts directly after the header

type spcifies the type of entry
there are several different valid types:

- 0: any file
- 1: early-init
- 2: part-list
- 3: fs-server
- 4: ahci-server

name specifies the offset into the initrd of the name of the entry,
and name\_len specifies the length of this string

the name string will be a valid utf-8 string

data specifies the offset into the initrd of the data of the entry,
and data\_len specifies the length, in bytes, of the data

name and data will always be 8 byte aligned
