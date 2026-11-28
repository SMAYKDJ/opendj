# Plano de Migração e Integração do openDJ v0.9.5 na Edição SMAYKDJ

> **Documento de Continuidade Técnica para Desenvolvedores e Agentes de IA (Claude Code / Antigravity)**  
> **Data:** 27 de Novembro de 2026 (Horário de Belém UTC−3)  
> **Status:** Branch de migração `smaykdj-v0.9.5` criada e tag de backup `smaykdj-pre-v0.9.5` versionada no GitHub.

---

## 1. Visão Geral e Objetivo

O repositório oficial do openDJ (`openwave-studio/opendj`) publicou a versão de teste **v0.9.5** (tag `v0.9.5`), que acumula mais de 60 PRs com grandes evoluções de arquitetura (sincronização V-Sync nas waveforms, Phase Meter nos decks, stems HQ a partir do playhead, mapeamentos nativos para DDJ-1000 com telas nos jogs, FLX10, Reloop, M32 e correções de áudio ASIO no Windows).

Nossa edição customizada (**SMAYKDJ**) contém um conjunto rico de funcionalidades proprietárias já desenvolvidas, testadas e commitadas na branch `smaykdj`.

**Objetivo deste plano:** Integrar todas as novidades do openDJ `v0.9.5` na branch `smaykdj-v0.9.5` **sem perder, regredir ou quebrar nenhuma das nossas customizações**, validando cada etapa com testes unitários e `cargo check`.

---

## 2. Estado do Versionamento no Git

Antes de qualquer alteração estrutural, os seguintes pontos de restauração foram criados e enviados ao remote:

* **Remote upstream:** `origin` ➔ `https://github.com/openwave-studio/opendj.git`
* **Remote do projeto:** `smaykdj` ➔ `https://github.com/SMAYKDJ/opendj.git`
* **Commit base estável:** `4fd5c3f` na branch `smaykdj`.
* **Tag de segurança:** `smaykdj-pre-v0.9.5` (push feito em `smaykdj`).
* **Branch de trabalho da migração:** `smaykdj-v0.9.5` (push inicial feito com tracking em `smaykdj/smaykdj-v0.9.5`).

> **Procedimento de Rollback:** Caso qualquer tentativa de merge ou integração falhe irrecuperavelmente, basta executar:
> ```bash
> git checkout smaykdj
> git reset --hard smaykdj-pre-v0.9.5
> ```

---

## 3. Inventário Completo das Customizações da Edição SMAYKDJ

Qualquer integração com o upstream **DEVE OBRIGATORIAMENTE PRESERVAR** os 7 módulos a seguir:

### Módulo 1: Battle Live (Transmissão RTMP Nativa + YouTube + 16:9 + Câmeras)
* **Arquivos exclusivos (não existem no upstream):**
  * `crates/live/` (Crate inteira: `compose.rs`, `director.rs`, `phones.rs`, `output.rs`, `session.rs`, `camera.html`, `Cargo.toml`).
  * `crates/app/src/live.rs`: Lógica de controle do painel Live, detecção de stream key no clipboard, inicialização do compositor e conexão RTMP via FFmpeg.
  * `crates/app/ui/live.slint`: Interface gráfica do painel Battle Live (seleção de formato 9:16 Vertical vs 16:9 Horizontal, seletor de destino TikTok / Instagram / YouTube, inputs de URL RTMP e Stream Key, prévia de vídeo ajustável).
  * `crates/app/ui/projector.slint`: Janela de projetor para lives e transmissão em telão.
* **Pontos de integração nos arquivos existentes:**
  * `crates/app/src/main.rs`: Inicialização do `LiveServer`, atalhos de abertura do painel Live e janela de projeção.
  * `crates/app/ui/main.slint`: Botão `LIVE` na barra superior (Top Bar), janela de projetor e overlay do painel.
  * `crates/app/src/prefs.rs`: Persistência das configurações de RTMP e formato preferido.
  * `Cargo.toml`: Declaração de `crates/live` no workspace.
  * `crates/app/Cargo.toml`: Dependência `opendj-live = { path = "../live" }`.

### Módulo 2: B2B Online (Colaboração Remota via STUN / Opus / TCP)
* **Arquivos:**
  * `crates/app/src/live_b2b.rs`: Descoberta de IP público via STUN, handshake UDP, canal de áudio estéreo com codec Opus em tempo real, sincronização de decks a 30 Hz.
  * `crates/engine/src/engine.rs`: Taps de áudio para envio B2B.

### Módulo 3: Armazenamento Portátil de Análise em Pendrive / USB
* **Arquivo:** `crates/engine/src/cache.rs`
* **Comportamento:**
  * Função `is_external_volume(path)` e `volume_root(path)`: Detecta se uma música está em volume removível (`/Volumes/...` no macOS ou `D:\`, `E:\`, etc. no Windows).
  * Salva automaticamente os arquivos de análise detalhada e waveforms em `openDJ/analysis/` e `openDJ/overviews/` diretamente na raiz do pendrive.
  * Lê do pendrive antes de ler do cache local da máquina.
  * Permite plug-and-play imediato de pendrives em outros computadores com openDJ sem necessitar de reanálise.
  * Teste unitário de proteção: `external_volume_analysis_stored_on_drive`.

### Módulo 4: Seleção Múltipla com Teclado e Mouse na Biblioteca
* **Arquivos:**
  * `crates/app/ui/library.slint`:
    * Propriedades `extending: bool`, âncora de seleção.
    * Funções `key-step(down, extend)`, `key-page(down, extend)`, `key-activate()`.
    * Suporte a `Shift + ↓`, `Shift + ↑`, `PageDown`, `PageUp`, `Enter`.
    * Suporte a clique de mouse com `Shift` (range contínuo) e `⌘/Ctrl` (seleção alternada).
  * `crates/app/ui/main.slint`:
    * Roteamento de eventos `Shift+Up`, `Shift+Down`, `PageUp/Down` para o browser quando o foco não for campo de texto.
  * `crates/app/src/browser/picks.rs`:
    * Métodos `chosen_rows()`, `chosen_tracks()`, `show_picks()`.

### Módulo 5: Análise em Lote (50, 100, 150) e Barra de Progresso Estilo Traktor
* **Arquivos:**
  * `crates/app/ui/library.slint`:
    * Barra de progresso horizontal nas linhas da lista: espera na fila (`analysing == 1`) e scanner animado lilás (`analysing == 2`) usando `animation-tick()`.
    * Botão inline `⚡ Analisar X faixas` no cabeçalho quando faixas estiverem selecionadas.
    * Menu de contexto: opções "Analisar próximas 50", "Analisar próximas 100", "Analisar próximas 150" (visíveis quando `LibraryEdit.unanalyzed-count >= 50/100/150`).
    * Dica explicativa nos menus: `"BPM · Chave · Waveform"`.
  * `crates/app/src/browser/analysis.rs`:
    * Callback `on_analyze_next(count)`.
    * Método `update_unanalyzed_count(&app)`.
    * Função `analyze()` com visibilidade `pub(crate)`.
  * `crates/app/src/browser/health.rs`:
    * Função `picked_rows()` com visibilidade `pub(crate)`.
  * `crates/app/src/shortcuts.rs` & `crates/script/src/verbs.rs`:
    * Verbo `analyze` registrado no openDJscript e acionável por atalho.

### Módulo 6: Botão de Deletar Hot Cues nos Decks & MIDI
* **Arquivos:**
  * `crates/app/ui/deck.slint`: Botão `×` visível no hover de cada pad de Hot Cue para deletar o ponto sem necessidade de combinações de botões.
  * `crates/controllers/src/apc_phrase.rs`: Integração do AKAI APC Key 25 mk2.

### Módulo 7: Configuração de Performance de Compilação
* **Arquivo:** `Cargo.toml`
* **Comportamento:**
  * O upstream colocou LTO pesado em algumas compilações. Mantivemos compilação paralela rápida no profile `dev` e `release`, reservando o `codegen-units = 1` e LTO fat apenas para o profile `commercial`.

---

## 4. Análise dos Arquivos em Conflito no Merge de `v0.9.5`

Quando o git tenta mesclar `v0.9.5` sobre `smaykdj`, os seguintes arquivos apresentam conflitos que devem ser resolvidos manualmente com critério:

| Arquivo Conflitante | O que mudou no v0.9.5 | O que temos no smaykdj | Como Resolver |
|---|---|---|---|
| `Cargo.toml` | Mudou versão para `0.9.5`, novas crates de estúdio | Adicionou `crates/live`, perfis de compilação | Aceitar as novas crates upstream + manter `crates/live` no workspace e os perfis otimizados de compilação |
| `crates/app/src/main.rs` | Refatoração da janela, novos menus Studio e Session | Inicialização do Battle Live, atalhos do Projetor e B2B | Preservar a inicialização do live e os callbacks do SMAYKDJ |
| `crates/app/src/browser.rs` | Refatoração de linhas, background panel | Chamada de `update_unanalyzed_count`, seleção múltipla | Manter as chamadas de seleção e contagem de não-analisadas |
| `crates/app/src/browser/analysis.rs` | Análise paralela, busca de letras (lyrics) | `on_analyze_next`, `update_unanalyzed_count`, `analyze` pub(crate) | Combinar a análise paralela do 0.9.5 com o callback `analyze_next` e a contagem de não-analisadas |
| `crates/app/ui/main.slint` | Top bar redesenhada com pílulas uniformes de 32px | Botão `LIVE`, janela de projetor, roteamento de Shift+Setas | Mover o botão `LIVE` para o novo padrão de pílulas da barra superior e manter os manipuladores de teclado |
| `crates/app/ui/library.slint` | Painéis laterais, novas colunas | Seleção contínua, barra de progresso animada, menu 50/100/150 | Inserir a barra de progresso no delegate de linha do novo `library.slint` e restaurar os menus e atalhos |
| `crates/app/ui/deck.slint` | Phase meter, novos layouts de pads | Botão `×` de remoção de hot cues no hover | Adicionar o botão `×` sobre os novos pads do deck |
| `crates/engine/src/cache.rs` | Refatoração de cache de stems em 3 arquivos | Cache portátil na raiz de pendrives USB (`openDJ/analysis`) | **CRÍTICO:** Manter integralmente a lógica de detecção de volume externo e fallback local |
| `crates/engine/src/stems.rs` | Layouts Stacked e 3 Lanes, cores Ember e Lilac | Melhorias de stems e taps de áudio | Aceitar as melhorias visuais do 0.9.5 mantendo os taps necessários para live |

---

## 5. Roteiro Passo a Passo de Execução

Se você estiver continuando este trabalho via Claude Code ou outro terminal, siga esta ordem estrita:

### Passo 1: Preparar o ambiente
```bash
cd /Users/smayk/dev/opendj
git checkout smaykdj-v0.9.5
git status # Garanta que a árvore está 100% limpa
```

### Passo 2: Executar o merge sem autocommit
```bash
git merge --no-commit --no-ff v0.9.5
```

### Passo 3: Resolver os conflitos arquivo por arquivo
1. **`Cargo.toml` e `Cargo.lock`:**
   - Deixar a versão como `0.9.5`.
   - Adicionar `"crates/live"` no `members` do workspace.
   - Manter os perfis `dev`, `release` rápidos e `commercial`.
2. **`crates/engine/src/cache.rs`:**
   - Manter as funções `is_external_volume`, `volume_root`, o salvamento em `openDJ/analysis/` e o teste `external_volume_analysis_stored_on_drive`.
   - Aceitar as melhorias do upstream para os 3 arquivos de cache de HQ stems.
3. **`crates/app/src/browser/analysis.rs` e `browser.rs`:**
   - Garantir que `pub(crate) fn analyze` esteja acessível.
   - Incluir `on_analyze_next` e a chamada `update_unanalyzed_count`.
4. **`crates/app/ui/library.slint`:**
   - Adicionar a propriedade `unanalyzed-count` e callback `analyze-next(count: int)` em `LibraryEdit`.
   - Aplicar a barra de progresso animada nas linhas com `r.analysing > 0`.
   - Adicionar as opções de 50/100/150 faixas no menu de contexto.
   - Manter `key-step`, `key-page`, `key-activate` e a seleção contínua com Shift.
5. **`crates/app/ui/deck.slint`:**
   - Recolocar o botão `×` no hover dos hot cues.
6. **`crates/app/ui/main.slint` e `crates/app/src/main.rs`:**
   - Adaptar o botão `LIVE` para o estilo de pílula da barra de ferramentas do 0.9.5.
   - Manter a inicialização do live e os atalhos de teclado.

### Passo 4: Compilar e Rodar os Testes
```bash
# 1. Compilação estática do Rust e Slint
cargo check -p opendj

# 2. Testes dos scripts e comandos
cargo test -p opendj-script

# 3. Teste do cache portátil de pendrive
cargo test -p opendj-engine cache

# 4. Teste da crate de live
cargo check -p opendj-live
```

### Passo 5: Commitar e Atualizar Documentação
```bash
git add -A
git commit -m "feat(upstream): integrar openDJ v0.9.5 preservando melhorias SMAYKDJ"
git push smaykdj smaykdj-v0.9.5
```

---

## 6. Checklist de Validação Final

Antes de considerar a migração concluída e mesclar para `smaykdj`:

- [ ] O app abre sem erros (`cargo run --bin opendj`).
- [ ] A barra superior tem o logo 1.0 e o botão **LIVE** funcional.
- [ ] O painel **Battle Live** abre e fecha perfeitamente com os formatos 9:16 e 16:9.
- [ ] O **Phase Meter** sob os decks funciona com batidas ao vivo.
- [ ] A navegação na biblioteca com `Shift + Setas` e mouse seleciona múltiplas faixas.
- [ ] O botão `[⚡ Analisar X faixas]` surge no topo e o menu com 50/100/150 funciona.
- [ ] A **barra de progresso animada** varre a linha da música enquanto ela é analisada.
- [ ] Ao analisar uma música em pendrive USB, os arquivos são salvos na raiz do pendrive em `openDJ/analysis/`.
- [ ] O botão `×` nos hot cues permite deletar pontos de cue com um clique de mouse.
- [ ] O arquivo `docs/smaykdj/CHANGELOG.md` registra a integração do v0.9.5.
