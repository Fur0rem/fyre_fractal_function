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
		gnuplot

		# perf
		linuxPackages.perf
		
		# Github
		git
	];
	RUSTC_VERSION = "stable";
	NIX_ENFORCE_NO_NATIVE = 0;

	shellHook = ''
		cargo install cargo-pgo
		cargo install flamegraph
	'';
}
