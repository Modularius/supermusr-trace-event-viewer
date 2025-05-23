{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    pipeline.url = "github:STFC-ICD-Research-and-Design/supermusr-data-pipeline";
    #pipeline.url = "/home/ubuntu/SuperMuSRDataPipeline?dir=supermusr-data-pipeline";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    pipeline,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          config.allowUnfree = true;
        };
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            nil
            nixd
            direnv
            valgrind-light
            cifs-utils
            nfs-utils
            hdf5_1_10
            kcat
          ];
          inputsFrom = [
            pipeline.devShell.${system}
          ];
        };
      }
    );
}
