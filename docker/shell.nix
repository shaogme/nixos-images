{ sources ? import ./npins
, system ? builtins.currentSystem
, pkgs ? import sources.nixpkgs { inherit system; config.allowUnfree = true; }
}:
let
  deps = import ./deps.nix { inherit pkgs; };
in
pkgs.mkShell {
  buildInputs = deps.all;

  # --- Environment Variables ---
  
  # 1. NIX_LD_LIBRARY_PATH (For nix-ld compatibility layer)
  NIX_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps.runtimeLibs;

  # 2. Hints for dynamic linker
  NIX_LD = pkgs.lib.fileContents "${pkgs.stdenv.cc}/nix-support/dynamic-linker";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps.runtimeLibs;
}
