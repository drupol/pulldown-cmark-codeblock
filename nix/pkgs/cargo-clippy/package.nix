{
  lib,
  cargo,
  clippy,
  pulldown-cmark-codeblock,
}:

pulldown-cmark-codeblock.overrideAttrs (oldAttrs: {
  nativeCheckInputs = (oldAttrs.nativeCheckInputs or [ ]) ++ [
    cargo
    clippy
  ];

  checkPhase = ''
    RUSTFLAGS="-Dwarnings" ${lib.getExe cargo} clippy
  '';
})
