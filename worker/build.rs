use std::env;
use std::path::PathBuf;

fn main() {
	let proto_files = vec![
		"./src/proto/memory/memory.proto",
		"./src/proto/compute/compute.proto",
		"./src/proto/network/network.proto",
		"./src/proto/storage/storage.proto",
		"./src/proto/helloworld.proto",
		"./src/proto/container/container.proto",
		"./src/proto/container/stats.proto",
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
