{ pkgs ? import (builtins.fetchTarball "https://github.com/NixOS/nixpkgs-channels/tarball/nixos-unstable") {},
 }:
let
  inherit (pkgs) writeTextFile;
  inherit (pkgs.stdenv) mkDerivation;
  inherit (pkgs.rustPlatform) buildRustPackage;

  shellcheckCmd = ''
    echo "=========================================================="
    echo "----> Shellchecking:"
    for file in `find . -name '*.sh'`; do
      echo "----> $file"
      ${pkgs.shellcheck}/bin/shellcheck -e SC1090 -e SC1091 -x $file
    done
  '';
in rec {
  ci = writeTextFile {
    name = "tests";
    text = ''
      ${lwnvulns.pkg}
      ${notate.pkgSrc}
      ${ported-notes.pkgSrc}
    '';
  };

  notate = rec {
    pkgSrc = mkDerivation {
      name = "notate-src";
      src = ./notate.sh;

      unpackPhase = ''
        cp $src .
      '';
      buildPhase = shellcheckCmd;

      installPhase = ''
        mkdir $out
        cp -r $src $out/notate.sh
      '';
    };
  };

  ported-notes = rec {
    pkgSrc = mkDerivation {
      name = "ported-notes-src";
      src = ./ported-notes.sh;

      unpackPhase = ''
        cp $src .
      '';
      buildPhase = shellcheckCmd;

      installPhase = ''
        mkdir $out
        cp -r $src $out/ported-note.sh
      '';
    };
  };


  lwnvulns = rec {
    dependencies = with pkgs; [
        perl
        pkgconfig
        openssl.dev
        zlib.dev
        curl.dev
    ];

    formatcheckCmd = ''
      echo "=========================================================="
      echo "----> Formatting:"
      for file in `find . -name '*.rs' -not -path '*/target/*'`; do # */ Hi emacs ...
        echo "----> $file"
        ${pkgs.rustfmt}/bin/rustfmt --write-mode=diff "$file"
      done
    '';

    shell = mkDerivation {
      name = "nixos-security-tools-shell";
      src = ./.;

      formatcheck = formatcheckCmd;

      buildInputs = dependencies ++ (with pkgs; [
        rustfmt
        rustc
        cargo
      ]);

      shellHook = ''
        cd lwnvulns;
      '';
    };

    pkg = buildRustPackage {
      name = "lwn-vuln";
      src = pkgSrc;

      depsSha256 = "19l6p7g8xy5hvy562saxaia1jxcbyrq657nnk7i055lla9i199p2";

      preBuild = formatcheckCmd;

      buildInputs = dependencies;
    };

    pkgSrc = mkDerivation {
      name = "nixos-security-tools-src";
      src = builtins.filterSource (
        path: type:
        let
          bpath = baseNameOf path;
        in !(
             ((builtins.substring 0 1 bpath) == ".")
          || (type == "symlink" && bpath == "result")
          || (type == "directory" && bpath == ".git")
          || (type == "directory" && bpath == "target")
          || (type == "file" && bpath == "db")
          || (type == "file" && bpath == "shell.nix")
          || (type == "file" && bpath == "default.nix")
        )
      ) ./lwnvulns;

      buildPhase = formatcheckCmd;

      installPhase = ''
        cp -r $src $out
      '';
    };
  };

  nixvulns = rec {
    dependencies = with pkgs; [
      notmuch
    ];

    shell = mkDerivation {
      name = "nixos-security-tools-shell-nixvulns";
      src = ./.;

      buildInputs = dependencies ++ (with pkgs; [
        rustfmt
        rustc
        cargo
        gdb
      ]);

      shellHook = ''
        cd nixvulns;
      '';
    };
  };

}
