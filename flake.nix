{
  description = "Nexus - A Rust-based plugin system for server management";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Use the latest nightly Rust toolchain
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # System dependencies needed for compilation and runtime
        buildInputs = with pkgs; [
          pkg-config    # For finding system libraries
          openssl       # For Discord webhook HTTPS connections  
          curl          # HTTP client functionality
          lsof          # System monitoring utilities
          gcc           # C compiler for potential C interop
        ];

        # Development and build tools
        nativeBuildInputs = with pkgs; [
          rustToolchain  # Rust compiler and tools
          
          # Development workflow tools
          bacon          # Hot-reloading development server
          cargo-watch    # File watching for cargo
          cargo-flamegraph # Performance profiling
          cargo-deny     # License/dependency checking
          cargo-audit   # Security vulnerability scanning
          
          # Binary analysis tools
          binutils       # nm, readelf, objdump for inspecting .so files
          
          # Code quality tools
          rustfmt        # Code formatting
          clippy         # Linting and static analysis
          
          # Nix development helpers
          nil            # Nix language server
          nixpkgs-fmt    # Nix code formatting
        ];
      in
      {
        # Development shell with all dependencies and tools
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          # Environment variables for development
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_LOG = "info";
          RUST_BACKTRACE = "1";

          # Shell startup message
          shellHook = ''
            echo "Welcome to Nexus development environment!"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Quick start:"
            echo "  bacon          - Hot-reloading development"
            echo "  cargo check    - Check code compilation"
            echo "  cargo test     - Run tests"
            echo "  cargo run      - Run main application"
            echo "  cargo build    - Build release version"
            echo ""
          '';
        };

        # Build the complete project as a package
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nexus";
          version = "0.1.0";

          src = ./.;
          cargoLock = ./Cargo.lock;
          
          inherit buildInputs nativeBuildInputs;

          # Disable tests if they require network access or special permissions
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A Rust-based plugin system for server management";
            homepage = "https://github.com/oestradiol/nexus";
            license = licenses.bsd3;
            maintainers = [ "Elaina <17bestradiol@proton.me>" ];
            platforms = platforms.linux ++ platforms.darwin;
          };
        };

        # Alias for the package
        packages.nexus = self.packages.${system}.default;

        # Nix code formatter
        formatter = pkgs.nixpkgs-fmt;
      });
}
