# Histórico de ajustes

Tudo o que mudou nesta edição em relação ao openDJ original, do mais novo
para o mais antigo. As datas seguem o horário de Belém (UTC−3).

## 2026-11-18

### Verificado
- Branch `smaykdj` compilado no MacBook Air (release, 11m52s) sem erros,
  incluindo o MAP MIDI completo e o progresso dos stems.
- Para abrir: `cargo run --release --bin opendj` (o projeto tem mais de um
  programa; sem `--bin` o `cargo run` não abre nada).

### Adicionado
- **Progresso da separação de stems em alta qualidade.** Enquanto o HTDemucs
  separa uma faixa, aparece uma pílula lilás embaixo do título do deck que se
  enche conforme avança ("STEMS 42%"). Se outra faixa estiver sendo separada
  antes, mostra "STEMS · QUEUED". O canto do pad STEMS passa a mostrar
  "HQ 42%" em vez de "HQ…".
  - Motor: o HTDemucs já avisava cada trecho processado; agora o openDJ guarda
    esse avanço por arquivo (`stems::separation_progress`).
  - Divisão do progresso: decodificar o áudio vai até 5%, o modelo até 90%,
    gravar os quatro stems o restante.
  - Arquivos: `crates/separation/src/htdemucs.rs`,
    `crates/engine/src/separation.rs`, `crates/engine/src/stems.rs`,
    `crates/app/src/main.rs`, `crates/app/ui/deck.slint`.
- **Documentação desta edição** em `docs/smaykdj/`, com a especificação da
  transmissão Battle Live para o TikTok.

### Corrigido
- **MAP MIDI não aparecia na tela.** Os componentes existiam em
  `mapmark.slint`, mas o botão não estava na barra superior, a barra lateral
  não era exibida e a lista de controles mapeados nunca era preenchida.
  Agora:
  - o botão MAP MIDI fica na barra superior, antes do REMOTE (fora do modo
    Player);
  - a barra lateral aparece à direita enquanto o modo de mapeamento está
    ligado;
  - `map_mode.rs` monta a lista a cada mudança: o que cada controle faz
    ("Play · Deck 1") e qual botão da controladora faz isso, pelo nome da
    peça no desenho da controladora ou, sem desenho, pela mensagem MIDI.
- Os comentários originais de `mapmark.slint` tinham sido apagados no Mac;
  foram mantidos nesta versão.

### Observado (já existia no openDJ 0.6.0)
- Mute por stem (pads STEMS, com ACAPELLA e INSTRUMENTAL), volume por stem
  (knobs do mixer no modo stems) e preparação de stems HQ em lote pela
  biblioteca (clique direito em várias faixas), com cache e limite de tamanho
  em Settings › Storage.

## 2026-11-15

### Adicionado
- **MAP MIDI (primeira versão).** Em `mapmark.slint`: o tipo `MappingEntry`
  (controle da controladora + função), as propriedades `entries` e
  `sidebar-open` em `MapMode`, o botão `MapMidiButton` e a barra lateral
  `MappingSidebar` ("CONTROLES MAPEADOS", com "Concluir mapeamento").

### Corrigido durante a compilação
- `viewport-height` está obsoleto no Slint 1.18: trocado por
  `content-height` na lista da barra lateral.
- `Theme.n-ffffff05` não existe no tema: trocado por `Theme.n-ffffff0a`
  (o menor tom transparente disponível é `n-ffffff08`).

### Ambiente
- Rust instalado e `source "$HOME/.cargo/env"` acrescentado ao `~/.zshrc`
  para o `cargo` funcionar em qualquer terminal novo.
