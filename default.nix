let pkgs = import (fetchTarball(
		"channel:nixos-unstable"
	)) {};
in
pkgs.mkShell {
	buildInputs = with pkgs; [
		# Rust
		rustup
		gcc
		coreutils-prefixed
		pkg-config
		# openssl
		# chromedriver
		gnuplot
		
		# Github
		git
	];
	RUSTC_VERSION = "stable";

	shellHook = ''
		cargo install cargo-pgo
	'';
}
