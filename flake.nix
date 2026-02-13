#    This program is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    This program is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with this program.  If not, see <https://www.gnu.org/licenses/>.

{
  description = "Entorno híbrido MCP: Python (Notas) + Rust (Qdrant/ML) con uv";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        # Librerías esenciales para compilación en Rust y Wheels de Python
        runtimeLibs = with pkgs; [
          stdenv.cc.cc.lib
          zlib
          libgcc.lib
          openssl
          pkg-config
        ];
      in {
        devShells.default = pkgs.mkShell {
          name = "mcp-hybrid-system";

          packages = with pkgs; [
            # --- Python Stack ---
            python313
            uv
            basedpyright
            ruff

            # --- Rust Stack ---
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            pkg-config
            openssl

            # --- Ops & Tools ---
            nodejs_24      # Útil si decides usar el inspector de MCP (npx)
            docker-compose
            curl
            jq
          ];

          env = {
            PYTHONUNBUFFERED = "1";
            # Importante para que Rust encuentre OpenSSL y las Wheels encuentren libgcc
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runtimeLibs}";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

          shellHook = ''
            echo "🦀 Rust + 🐍 Python: Entorno Nix cargado para MCP Híbrido."
            echo "📍 Cargo version: $(cargo --version)"
            echo "📍 Python (uv) listo para gestionar el servidor de notas locales."
            echo "🚀 Qdrant Status: $(docker ps --filter "name=qdrant" --format "{{.Status}}" || echo 'Qdrant container not running')"
          '';
        };
      });
}
