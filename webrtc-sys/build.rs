use std::{
	env,
	io::Error,
	iter::once,
	path::{Path, PathBuf},
	process::Command,
};

fn run(cmd: &mut Command) -> Result<(), Error> {
	let mut child = cmd.spawn()?;
	let res = child.wait()?;
	if res.success() {
		Ok(())
	} else {
		Err(Error::other(format!("Failed: {cmd:?} returned {res}")))
	}
}

fn main() -> Result<(), Error> {
	// Ask cxx to build the c++ bindings, but don't actually compile them: we will use gn/ninja to compile them
	// NOTE: We changed our work dir to .. so we need to prefix our file paths with webrtc-sys
	let _ = cxx_build::bridges(["src/lib.rs"]);

	// TODO: We need a BUILD.gn file that rtc_dynamic_library's the cxx generated c++ and deps on the webrtc api stuff...

	// Change to parent directory
	env::set_current_dir(Path::new(".."))?;

	// Initialize git submodules (depot_tools)
	run(Command::new("git").args(["submodule", "update", "--init"]))?;

	// Add depot_tools to PATH
	let dt_path = Path::new("depot_tools").canonicalize()?;
	let current_path = env::var_os("PATH").ok_or(Error::other("Where's your PATH?"))?;
	let new_path = env::join_paths(once(dt_path).chain(env::split_paths(&current_path)))
		.map_err(Error::other)?;
	env::set_var("PATH", new_path);

	// Sync
	run(Command::new("gclient").args([
		"sync",
		"--nohooks", // OK Google, explain to me why "tabs are evil", but nohooks and no-history is fine?
		"--no-history",
	]))?;

	let profile = env::var("PROFILE").map_err(Error::other)?;
	let profile_dir = Path::new("out").join(&profile);

	// Generate
	run(Command::new("gn")
		.current_dir("src")
		.arg("gen")
		.arg(&profile_dir))?;

	// Compile
	run(Command::new("ninja")
		.current_dir(Path::new("src").join(profile_dir))
		.arg(":webrtc"))?;

	// Link against webrtc
	println!(
		"cargo::rustc-link-search=native={}",
		PathBuf::from_iter(["src", "out", &profile, "obj"])
			.to_str()
			.ok_or(Error::other("yikes"))?
	);
	println!("cargo::rustc-link-lib=webrtc");

	Ok(())
}
