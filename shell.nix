let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/refs/tags/24.05.tar.gz")) {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell rec {
  name = "chip8-rust";

  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    rust-analyzer
    SDL2
  ];
  SDL_VIDEO_X11_VISUALID="0x022";
}
