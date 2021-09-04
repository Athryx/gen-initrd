use clap::clap_app;

fn main()
{
	let matches = clap_app!(("gen-initrd") =>
		(version: "0.1.0")
		(about: "Simple utility to generate initrd image for the aurora kernel")
		(@arg ("early-init"): -i --init <EXECUTABLE> "First executable spawned by kernel which is responsible for mounting the root filesytem and spawning the init process")
		(@arg ("part-list"): -p --("part-list") <FILE> "File read by early-init which describes which filesytem drivers to use for which partitions and where to mount them")
		(@arg ("fs-server"): -f --fs <EXECUTABLE> "Filesystem server which filesytem drivers will connect to")
		(@arg ("ahci-server"): -a --ahci <EXECUTABLE> "Ahci server to allow filesytem drivers to communicate with drives")
		(@arg ("ext2-server"): -e --ext2 [EXECUTABLE] "ext2 filesytem driver")
		(@arg files: [FILE] ... "additional files to include in initrd")
	).get_matches();
}
