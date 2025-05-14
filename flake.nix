# https://github.com/vimjoyer/shells2-video
{
  description = "cargo dev env";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
    in
    {
      devShells."x86_64-linux".default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          cargo-watch
          just
          tmux
          sqlite
          nodejs
          sqlx-cli
          pkg-config
          openssl
        ];

        RUST_BACKTRACE = 1;
      };
    };
}
