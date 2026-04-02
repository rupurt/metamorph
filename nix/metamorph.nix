{ lib, rustPlatform, pkg-config, ... }:

let
  cargoToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = "metamorph";
  version = cargoToml.workspace.package.version;

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  cargoBuildFlags = [ "-p" "metamorph-cli" ];
  cargoTestFlags = [ "-p" "metamorph" "-p" "metamorph-cli" ];

  nativeBuildInputs = [
    pkg-config
  ];

  meta = with lib; {
    description = "Model format conversion utility for local-first AI runtimes";
    homepage = "https://github.com/rupurt/metamorph";
    license = licenses.mit;
    mainProgram = "metamorph";
    maintainers = [ ];
  };
}
