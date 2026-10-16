{
  description = "greenhouse_backend: service binaries + devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      # rust-toolchain.toml pins the nightly channel; pick the most recent
      # nightly that ships every component we ask for.
      toolchain = pkgs.rust-bin.selectLatestNightlyWith (
        t:
        t.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        }
      );
    in
    {
      # The six service binaries (the workspace's default-members): web_api,
      # scripting_api, auth_service, data_storage_service, device_service,
      # scripting_service. Deployed by pinning a git tag of this flake.
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "greenhouse-backend";
        version = self.shortRev or self.dirtyShortRev or "dirty";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs; [
          openssl
          postgresql # libpq for diesel's migrations
        ];
        env.OPENSSL_NO_VENDOR = "1";

        # integration-tests need Docker (testcontainers).
        doCheck = false;
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ pkg-config ];

        buildInputs = with pkgs; [
          openssl # openssl-sys (reqwest/native-tls, sentry)
          postgresql # pq-sys: libpq + pg_config
        ];

        packages =
          [ toolchain ]
          ++ (with pkgs; [
            just # task runner (justfile)
            typos # CI: typo check
            cargo-sort # CI: cargo sort --workspace --grouped
            cargo-nextest
            bacon
            (diesel-cli.override {
              sqliteSupport = false;
              mysqlSupport = false;
            })
          ]);

        # Each service owns its own database (auth/data/device), so DATABASE_URL
        # is per-service — e.g. postgres://admin:password@localhost:5432/auth —
        # and is left to the caller rather than set here.
        env.OPENSSL_NO_VENDOR = "1"; # never let openssl-sys vendor its own copy
      };
    };
}
