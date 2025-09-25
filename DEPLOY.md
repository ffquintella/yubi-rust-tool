# Guia de Deploy — ykchalresp

Este guia explica como compilar, instalar e disponibilizar o binário `ykchalresp` em sistemas Linux e macOS. O projeto é um binário Rust que fala diretamente com uma YubiKey para HMAC‑SHA1 challenge‑response, e também oferece um modo de simulação em software.

## Pré‑requisitos
- Rust (toolchain estável) e `cargo` instalados
- Acesso de leitura ao dispositivo YubiKey (ver regras udev no Linux)
- Opcional: privilégios de administrador para instalar em diretórios do sistema

Dica: para builds reproduzíveis, use `--locked` para respeitar o `Cargo.lock`.

## Compilar
- Debug: `cargo build --bin ykchalresp`
- Release (recomendado): `cargo build --release --bin ykchalresp`

Saídas:
- Debug: `target/debug/ykchalresp`
- Release: `target/release/ykchalresp`

Verifique a versão: `target/release/ykchalresp -V`

## Instalar o binário
Existem duas formas principais:

- Usuário (sem sudo), via Cargo:
  - `cargo install --path .`  (instala em `~/.cargo/bin/ykchalresp`)

- Sistema (cópia manual do artefato release):
  - `sudo install -m 0755 target/release/ykchalresp /usr/local/bin/ykchalresp`

Garanta que o diretório de instalação está no `PATH`.

## Instalar a página de manual (opcional)
Este repositório inclui `man/ykchalresp.1` e um utilitário `xtask` para instalar/desinstalar a manpage:

- Instalar:
  - `cargo run --bin xtask -- man-install`
- Desinstalar:
  - `cargo run --bin xtask -- man-uninstall`

Variáveis de ambiente suportadas:
- `PREFIX` (padrão: `/usr/local`), ex.: `PREFIX=/opt/yk cargo run --bin xtask -- man-install`
- `DESTDIR` (raiz de staging, útil para empacotamento), ex.: `DESTDIR=./pkgroot cargo run --bin xtask -- man-install`

O destino padrão é: `PREFIX/share/man/man1/ykchalresp.1`.

## Linux: permissões para acessar a YubiKey
Para permitir que usuários não‑root acessem o dispositivo via HID/hidraw, adicione regras udev. Exemplo comum (ajuste o grupo conforme sua distro, ex.: `plugdev` ou `users`):

```
# /etc/udev/rules.d/70-yubikey.rules
SUBSYSTEMS=="usb", ATTRS{idVendor}=="1050", MODE="0660", GROUP="plugdev"
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1050", MODE="0660", GROUP="plugdev"
```

Depois, recarregue udev e reconecte a YubiKey:
- `sudo udevadm control --reload-rules`
- `sudo udevadm trigger`
- Desconecte/conecte a YubiKey

Em macOS, nenhuma configuração adicional costuma ser necessária.

## Teste rápido (hardware)
- Slot 2 (padrão): `ykchalresp "teste"`
- Slot 1: `ykchalresp -1 "teste"`
- Hex de entrada/saída: `echo -n 7465737465 | xxd -r -p | ykchalresp -x`

O comando imprime a resposta em modhex (padrão) ou em hex quando `-x` é usado.

## Configuração do modo simulado (sem hardware)
Para uso sem YubiKey (`-s`), defina a chave HMAC (hex) via variável de ambiente ou arquivo:

- Variáveis (sobrepõem arquivos):
  - Slot 1: `export YKCHALRESP_SLOT1_KEY=001122...`
  - Slot 2: `export YKCHALRESP_SLOT2_KEY=001122...`
- Arquivos:
  - `~/.config/ykchalresp/slot1.key` ou `slot2.key` contendo a chave em hex

Exemplo: `ykchalresp -s -x "74657374"`

## Empacotamento (opcional)
- Staging com `DESTDIR` para empacotes `.deb`/`.rpm`:
  - Binário: `install -Dm0755 target/release/ykchalresp "$DESTDIR/usr/local/bin/ykchalresp"`
  - Manpage: `DESTDIR="$DESTDIR" cargo run --bin xtask -- man-install`
- Para tarball simples:
  - `tar -C target/release -czf ykchalresp-linux-x86_64.tar.gz ykchalresp`

## Desinstalação
- Via Cargo: `cargo uninstall ykchalresp`
- Manual: `sudo rm -f /usr/local/bin/ykchalresp` e (opcional) `sudo rm -f /usr/local/share/man/man1/ykchalresp.1`

## Solução de problemas
- Sem acesso ao dispositivo: confira as regras udev (Linux) e re‑logue.
- Falha ao encontrar YubiKey: reconecte o dispositivo, teste outra porta USB.
- Resposta vazia/erro em simulação: valide a chave HMAC em hex (tamanho par, apenas 0‑9 a‑f).
- Recompile com logs mais verbosos: `cargo build --release --locked` e rode novamente.
