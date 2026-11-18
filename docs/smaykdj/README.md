# openDJ — edição SMAYKDJ

Este repositório é um fork do [openDJ](https://github.com/openwave-studio/opendj)
(GPL v2) com ajustes próprios para tocar e transmitir. O código original
continua igual; as mudanças ficam no branch `smaykdj` e estão listadas em
[CHANGELOG.md](CHANGELOG.md).

## O que esta edição acrescenta

| Recurso | O que faz | Estado |
| --- | --- | --- |
| MAP MIDI | Botão na barra superior que liga o modo de mapeamento e abre uma barra lateral com todos os controles já mapeados | Pronto, aguardando teste no Mac |
| Progresso dos stems HQ | Mostra no deck quanto falta para a separação em alta qualidade (HTDemucs) terminar | Pronto, aguardando teste no Mac |
| Battle Live (TikTok) | Transmissão vertical para a live do TikTok com câmeras de celular e diretor de cenas automático | Especificação: [live-tiktok.md](live-tiktok.md) |

## Base

- openDJ 0.6.0, commit `34b76f0` do projeto original.
- Testado em MacBook Air (Apple Silicon), macOS, Rust estável.

## Como compilar no Mac

```bash
git clone https://github.com/SMAYKDJ/opendj.git ~/dev/opendj
cd ~/dev/opendj
git checkout smaykdj
cargo build --release -p opendj
cargo run --release
```

A primeira compilação depois de mudar arquivos `.slint` demora bastante
(perto de 15 minutos); as seguintes levam segundos.

Se o terminal disser `cargo: command not found`, rode
`source "$HOME/.cargo/env"` uma vez, ou acrescente essa linha ao `~/.zshrc`.

## Branches

- `smaykdj`: a edição, com todos os ajustes (branch padrão deste fork).
- `main`: o openDJ original, sem mudanças, para comparar e atualizar.

## Licença

GPL v2, a mesma do openDJ. Veja [LICENSE](../../LICENSE).
