with import <nixpkgs> {};

let nightly = makeRustPlatform {
	# inherit (rustChannels.nightly) rustc cargo;
	# cargo = pkgs.extraRust.xargo;
	rustc = pkgs.rustup;
	cargo = pkgs.rustup;
};
in
nightly.buildRustPackage {
	
	name = "actos";
	version = "0.1.0";
	src = ./.;

	RUSTFLAGS = "-L ${pkgs.rustChannels.nightly.rust-std}/lib/rustlib/${stdenv.targetPlatform.config}/lib/";
	cargoSha256 = "1krj52zix6z21npd8ck2r8ac9dc1c1nl971p0wswfig4y97by34h";

	buildInputs = [ pkgs.extraRust.xargo rustChannels.nightly.rust-src pkgs.extraRust.bootimage ];

    doCheck = false;
}