{ inputs, withSystem, ... }:
{
  imports = [
    inputs.pkgs-by-name-for-flake-parts.flakeModule
  ];

  perSystem = {
    pkgsDirectory = ../pkgs;
  };

  flake = {
    overlays.default =
      _final: prev:
      withSystem prev.stdenv.hostPlatform.system (
        { config, ... }:
        {
          inherit (config.packages) pulldown-cmark-codeblock;
        }
      );
  };
}
