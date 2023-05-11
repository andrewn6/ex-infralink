use std::env;
use std::path::PathBuf;

fn main() {
	let proto_files = vec![
		"./proto/memory/memory.proto",
		"./proto/compute/compute.proto",
		"./proto/network/network.proto",
		"./proto/storage/storage.proto",
		"./proto/helloworld.proto",
		"./proto/container/container.proto",
	];

	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

	let proto_paths: Vec<PathBuf> = proto_files.iter().map(PathBuf::from).collect();

	tonic_build::configure()
		.build_server(true)
		.file_descriptor_set_path(out_dir.join("greeter_descriptor.bin"))
		.out_dir("./src")
		.compile(&proto_paths, &["."])
		.unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

	for proto_file in proto_files {
		println!("cargo:rerun-if-changed={}", proto_file);
	}
}
