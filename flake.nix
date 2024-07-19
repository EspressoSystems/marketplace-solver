{
  description = "marketplace solver";

  nixConfig = {
    extra-substituters = [
      "https://espresso-systems-private.cachix.org"
      "https://nixpkgs-cross-overlay.cachix.org"
    ];
    extra-trusted-public-keys = [
      "espresso-systems-private.cachix.org-1:LHYk03zKQCeZ4dvg3NctyCq88e44oBZVug5LpYKjPRI="
      "nixpkgs-cross-overlay.cachix.org-1:TjKExGN4ys960TlsGqNOI/NBdoz2Jdr2ow1VybWV5JM="
    ];
  };

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nixpkgs-cross-overlay.url =
    "github:alekseysidorov/nixpkgs-cross-overlay";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.flake-compat.url = "github:edolstra/flake-compat";
  inputs.flake-compat.flake = false;

  inputs.pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    , nixpkgs-cross-overlay
    , flake-utils
    , pre-commit-hooks
    , fenix
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      # node=error: disable noisy anvil output
      RUST_LOG = "info,libp2p=off,isahc=error,surf=error,node=error";
      RUST_BACKTRACE = 1;
      ASYNC_FLAGS = " --cfg async_executor_impl=\"async-std\" --cfg async_channel_impl=\"async-std\" ";
      RUSTFLAGS = "${ASYNC_FLAGS}";
      RUSTDOCFLAGS = ASYNC_FLAGS;
      # Use a distinct target dir for builds from within nix shells.
      CARGO_TARGET_DIR = "target/nix";
      rustEnvVars = { inherit RUST_LOG RUST_BACKTRACE RUSTFLAGS RUSTDOCFLAGS CARGO_TARGET_DIR; };

      overlays = [
        (import rust-overlay)
      ];
      pkgs = import nixpkgs { inherit system overlays; };
      crossShell = { config }:
        let
          localSystem = system;
          crossSystem = {
            inherit config;
            useLLVM = true;
            isStatic = true;
          };
          pkgs = import "${nixpkgs-cross-overlay}/utils/nixpkgs.nix" {
            inherit overlays localSystem crossSystem;
          };
        in
        import ./cross-shell.nix
          {
            inherit pkgs;
            envVars = rustEnvVars;
          };
    in
    with pkgs; {
      checks = {
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            cargo-fmt = {
              enable = true;
              description = "Enforce rustfmt";
              entry = "cargo fmt --all";
              types_or = [ "rust" "toml" ];
              pass_filenames = false;
            };
            cargo-sort = {
              enable = true;
              description = "Ensure Cargo.toml are sorted";
              entry = "cargo sort -g -w";
              types_or = [ "toml" ];
              pass_filenames = false;
            };
            cargo-clippy = {
              enable = true;
              description = "Run clippy";
              entry = "cargo clippy --all --all-features";
              types_or = [ "rust" "toml" ];
              pass_filenames = false;
            };
            nixpkgs-fmt.enable = true;
          };
        };
      };
      devShells.default =
        let
          stableToolchain = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [ "rustfmt" "clippy" "llvm-tools-preview" "rust-src" ];
          };
          # nixWithFlakes allows pre v2.4 nix installations to use
          # flake commands (like `nix flake update`)
          nixWithFlakes = pkgs.writeShellScriptBin "nix" ''
            exec ${pkgs.nixFlakes}/bin/nix --experimental-features "nix-command flakes" "$@"
          '';
        in
        mkShell (rustEnvVars // {
          buildInputs = [
            # Rust dependencies
            pkg-config
            openssl
            curl
            stableToolchain

            # Rust tools
            cargo-audit
            cargo-edit
            cargo-sort
            fenix.packages.${system}.rust-analyzer

            # Tools
            nixWithFlakes
            nixpkgs-fmt
            entr

          ] ++ lib.optionals stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.SystemConfiguration ]
          ++ lib.optionals (!stdenv.isDarwin) [ cargo-watch ] # broken on OSX
          ;
          shellHook = ''
            # Add node binaries to PATH
            export PATH="$PWD/node_modules/.bin:$PATH"
            # Prevent cargo aliases from using programs in `~/.cargo` to avoid conflicts
            # with rustup installations.
            export CARGO_HOME=$HOME/.cargo-nix
            export PATH="$PWD/$CARGO_TARGET_DIR/release:$PATH"
          '' + self.checks.${system}.pre-commit-check.shellHook;
          RUST_SRC_PATH = "${stableToolchain}/lib/rustlib/src/rust/library";
        });
      devShells.crossShell =
        crossShell { config = "x86_64-unknown-linux-musl"; };
      devShells.armCrossShell =
        crossShell { config = "aarch64-unknown-linux-musl"; };
      devShells.nightly =
        let
          toolchain = pkgs.rust-bin.nightly.latest.minimal.override {
            extensions = [ "rustfmt" "clippy" "llvm-tools-preview" "rust-src" ];
          };
        in
        mkShell (rustEnvVars // {
          buildInputs = [
            # Rust dependencies
            pkg-config
            openssl
            curl
            toolchain
          ];
        });


      devShells.rustShell =
        let
          stableToolchain = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [ "rustfmt" "clippy" "llvm-tools-preview" "rust-src" ];
          };
        in
        mkShell (rustEnvVars // {
          buildInputs = [
            # Rust dependencies
            pkg-config
            openssl
            curl
            stableToolchain
          ];
        });
    });
}
