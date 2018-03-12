#!/bin/sh

nix-shell -E 'with import <nixpkgs> {}; pkgsi686Linux.stdenv.mkDerivation { name = "dummy"; buildInputs = []; }' --run "cargo build --target i686-unknown-linux-gnu"