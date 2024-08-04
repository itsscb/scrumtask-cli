{
  description = "Example Rust development environment for Zero to Nix";

  # Flake inputs
  inputs = {
    # nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";
    rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
    # cargo2nix.url = "github:cargo2nix/cargo2nix/";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  # Flake outputs
  outputs = { self, nixpkgs, rust-overlay}:
    let
      # Overlays enable you to customize the Nixpkgs attribute set
      overlays = [
        # Makes a `rust-bin` attribute available in Nixpkgs
        (import rust-overlay)
        # Provides a `rustToolchain` attribute for Nixpkgs that we can use to
        # create a Rust environment
        (self: super: {
          rustToolchain = super.rust-bin.stable.latest.default;
        })
      ];

      # Systems supported
      allSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      # rustTarget = nixpkgs.pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
      #       extensions = [ "rust-src" "rustup" "rust-analyzer" "rust-std" ];
      #       targets = [ "x86_64-unknown-linux-gnu" ];
      #     });

      # Helper to provide system-specific attributes
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
      });
    in
    {
      # Development environment output
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          # shellHook = ''
          #   rustup target add wasm32-unknown-unknown
          # '';
          # The Nix packages provided in the environment
          packages = (with pkgs; [
            # The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
            # rustdoc, rustfmt, and other tools.
            rust-analyzer
            clippy
            rustToolchain
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
        };
      });
    };
}
