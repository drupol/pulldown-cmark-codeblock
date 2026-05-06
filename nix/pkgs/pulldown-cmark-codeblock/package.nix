{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "pulldown-cmark-codeblock";
  version = "0.1.0";

  __structuredAttrs = true;

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../Cargo.toml
      ../../../Cargo.lock
      ../../../src
    ];
  };

  cargoHash = "sha256-QUovFgKMGN4H6SeybN3gO/bnTl6nHOEkDnYLT0Kb8yw=";
}
