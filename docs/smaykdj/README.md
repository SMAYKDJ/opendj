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
| Battle Live (TikTok) | Botão LIVE na barra superior: transmissão vertical para a live do TikTok com câmeras de celular, projetor para TikTok LIVE Studio, diretor de cenas automático e gravação | Pronto, aguardando teste no Mac: [live-tiktok.md](live-tiktok.md) |
| B2B Online Remoto | Sessão P2P com Código de Sala para 2 DJs tocarem juntos pela internet com waveforms ao vivo, áudio Opus, câmera remota e live conjunta no TikTok | Pronto: [manual](manual-b2b-online.md) · [tour rápido](tour-b2b-online.md) · [roadmap](b2b-online.md) |
| Licenciamento & Proteção Comercial | Blindagem de binário (strip/LTO), anti-debug, hardware ID (macOS), chaves Ed25519 e webhook de pagamento automático | Pronto (Modo Soft/Testes Livres ativo): [documentação](licensing-commercial.md) |
| RTMP Direto (TikTok & Instagram) | Transmissão H.264+AAC direto do openDJ para TikTok e Instagram Live Producer (sem exigência de 1.000 seguidores), seletor de plataforma, chave no Keychain | Pronto: [guia RTMP](guia-transmissao-rtmp.md) |
| Modo Solo documentado | Guia completo de câmeras iguais no modo Solo, posicionamento, cenas, transmissão | Pronto: [guia Solo](guia-solo-modo.md) |
| Arquitetura Técnica | Referência completa: pipeline, crates, protocolos, diagramas, comandos dev | Pronto: [arquitetura](arquitetura-tecnica.md) |

## Base

- openDJ 0.6.0, commit `34b76f0` do projeto original.
- Testado em MacBook Air (Apple Silicon), macOS, Rust estável.

## Como compilar no Mac

```bash
git clone https://github.com/SMAYKDJ/opendj.git ~/dev/opendj
cd ~/dev/opendj
git checkout smaykdj
cargo build --release -p opendj
cargo run --release -p opendj --bin opendj
```

A primeira compilação depois de mudar arquivos `.slint` demora bastante
(perto de 12 minutos num MacBook Air); as seguintes levam segundos.

O `--bin opendj` é necessário: o projeto tem quatro programas (`opendj`,
`opendj-plugin-host`, `phrase-bench`, `stem-bench`) e sem ele o `cargo run`
não sabe qual abrir. Também dá para abrir direto: `./target/release/opendj`.
Sem o `-p opendj`, o `cargo` recompila o projeto inteiro com outra
configuração (vários GB a mais em disco). Depois de compilado, o jeito mais
rápido de abrir é `./target/release/opendj`, que não compila nada.

Se o terminal disser `cargo: command not found`, rode
`source "$HOME/.cargo/env"` uma vez, ou acrescente essa linha ao `~/.zshrc`.

## Battle Live precisa do ffmpeg

O vídeo é codificado pelo ffmpeg, que não vem junto com o openDJ. No Mac:

```bash
brew install ffmpeg
```

## Branches

- `smaykdj`: a edição, com todos os ajustes (branch padrão deste fork).
- `main`: o openDJ original, sem mudanças, para comparar e atualizar.

## Licença

GPL v2, a mesma do openDJ. Veja [LICENSE](../../LICENSE).
