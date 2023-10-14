let
  pkgs = import <nixpkgs> {};
in with pkgs; stdenv.mkDerivation rec {
  name = "vkrunner";
  nativeBuildInputs = [ meson ninja pkg-config rustc rust-bindgen ];
  buildInputs = [ vulkan-loader ];
  LD_LIBRARY_PATH = lib.makeLibraryPath [ vulkan-loader ];
}
