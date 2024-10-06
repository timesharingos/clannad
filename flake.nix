{
  description = "timesharingos/clannad";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = { self, nixpkgs, ... }: {

    devShells."x86_64-linux".default = let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    pkgs.mkShellNoCC {
	packages = [pkgs.wget
		    pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.gcc
		    pkgs.openssl pkgs.pkg-config
			pkgs.gh
		   ];
    };
  };
}
