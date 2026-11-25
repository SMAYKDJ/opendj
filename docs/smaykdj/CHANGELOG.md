# Histórico de ajustes

Tudo o que mudou nesta edição em relação ao openDJ original, do mais novo
para o mais antigo. As datas seguem o horário de Belém (UTC−3).

## 2026-11-25

### Adicionado
- **Transmissão RTMP direta (sem TikTok LIVE Studio):**
  - URL do servidor TikTok (`rtmp://push.tiktok.com/live/`) pré-preenchida
    automaticamente ao abrir o painel LIVE pela primeira vez.
  - Instrução simplificada na UI: "TikTok → + → Live → Stream de PC".
  - Aviso de ffmpeg faltando mais claro: "⚠ ffmpeg não encontrado. Cole no Terminal: brew install ffmpeg".
- **Documentação completa (3 novos documentos em `docs/smaykdj/`):**
  - [`guia-transmissao-rtmp.md`](guia-transmissao-rtmp.md) — Guia de RTMP para
    TikTok, YouTube, Instagram, configuração de qualidade, troubleshooting e
    tabela de servidores de todas as plataformas.
  - [`guia-solo-modo.md`](guia-solo-modo.md) — Modo Solo com câmeras de mesmo
    tamanho: layout da tela 9:16 detalhado, como configurar celulares, cenas
    disponíveis, diretor automático e transmissão ao vivo.
  - [`arquitetura-tecnica.md`](arquitetura-tecnica.md) — Referência técnica
    completa: crates, pipeline de codificação, sistema de câmeras, compositor
    tiny-skia, B2B Online, licenciamento Ed25519, anti-tamper, perfis de
    compilação, diagrama de dependências e comandos de desenvolvimento.
- **README atualizado** com links para os novos documentos.

### Verificado
- `cargo check -p opendj` passou (exit 0) após todas as mudanças.
  Apenas warnings pré-existentes do Slint (binding loop / `viewport-height`
  deprecation) — nenhum erro novo introduzido.

---

## 2026-11-24 (noite)

### Adicionado
- **Arquitetura Comercial, Anti-Pirataria e Licenciamento Criptográfico:**
  - **Perfil de Compilação Blindado (`[profile.release]`):** ativado `strip = true` (elimina todos os símbolos e caminhos de debug do binário compilado), `lto = "fat"`, `codegen-units = 1` e `panic = "abort"` para máxima proteção contra engenharia reversa e desmontagem por IDA Pro / Ghidra.
  - **Módulo de Segurança e Anti-Tampering (`crates/app/src/security.rs`):**
    - Detecção de depuradores ativos (`is_debugger_present`): monitora processos pais e injeção de dylibs (`DYLD_INSERT_LIBRARIES`, Frida, Cycript).
    - Proteção `PT_DENY_ATTACH` do kernel macOS para impedir a anexação de depuradores ao executável.
    - Decodificador ofuscado de strings em memória (`ObfStr`) para evitar strings sensíveis em texto puro no binário (.rodata).
  - **Hardware Fingerprint — Machine ID Único (`crates/app/src/license.rs`):**
    - Identificador persistente baseado no serial do hardware (`IOPlatformSerialNumber` / UUID) com hash SHA-256 no formato `MACH-XXXX-XXXX-XXXX-XXXX`.
    - Comando CLI embutido `./target/release/opendj --machine-id` para consulta rápida no terminal.
  - **Validador Criptográfico de Licença com Curva Elíptica Ed25519:**
    - Criptografia assimétrica padrão bancário (via crate `ring`): validação matemática offline de licenças digitais sem depender de conexão com internet.
    - Suporte a licenças assinadas no formato `ODJ1.<PAYLOAD_B64>.<ASSINATURA_B64>`.
    - Persistência automática em `~/Library/Application Support/openDJ/license.json`.
    - Comandos administrativos embutidos: `--keygen` (geração de par de chaves) e `--create-license` (geração de licença para clientes).
  - **Interface no Painel LIVE:**
    - Seção "LICENÇA & PRODUTO" com exibição do Hardware ID e botão "Copiar ID".
    - Campo de teste e ativação de licença com feedback em tempo real.
  - **Modo Soft / Testes Livres (`ENFORCE_LICENSE = false`):**
    - Todas as funcionalidades do openDJ (Battle Live vertical, projetor TikTok, B2B Online) permanecem 100% liberadas e operacionais durante a fase de testes, exibindo a pílula "TESTES LIBERADOS".
    - A obrigatoriedade da licença fica pendente até a conclusão dos testes pelo usuário, bastando mudar a flag para iniciar a comercialização estrita.
  - **Documentação e Webhook Serverless de Pagamento Automático:**
    - Criado [licensing-commercial.md](licensing-commercial.md) com especificação completa e código de webhook pronto para Cloudflare Workers / Node.js integrando Mercado Pago (PIX instantâneo) e Stripe com envio automático de chave por e-mail em menos de 10 segundos.

## 2026-11-24 (tarde)

### Adicionado
- **B2B Online Remoto (Signaling e NAT Traversal via STUN RFC 5389):**
  - Implementado cliente STUN nativo em `live_b2b::discover_public_endpoint`: descobre automaticamente o IP público e mapeamento de porta externa via servidores STUN do Google e Cloudflare sem bibliotecas externas.
  - Ao criar a sala, o anfitrião consulta o STUN em segundo plano e exibe o IP público na UI para conexão remota através da internet.
  - O botão "Copiar" gera uma mensagem completa contendo o código de sala e o IP público da internet para compartilhamento via WhatsApp/chat.
  - O DJ convidado pode digitar ou colar diretamente o IP ou IP:Porta (ex: `201.86.12.34:7498`), conectando instantaneamente pela internet sem esperar o timeout de descoberta local.
- **B2B Online Remoto (Transmissão de Áudio Estéreo com Codec Opus):**
  - Implementado streaming de áudio contínuo de ultrabaixa latência (20 ms por pacote @ 128 kbps) utilizando o codec Opus (`B2bMessage::AudioOpus`).
  - Cada instância extrai o áudio estéreo do master feed local em tempo real, comprime via `opus::Encoder` e transmite pela rede.
  - O receptor descomprime via `opus::Decoder` e alimenta o buffer de áudio do compositor vertical (`Session::push_remote_audio`), que mescla perfeitamente os dois DJs tanto para a live no TikTok LIVE Studio quanto para as gravações em MP4.
- **B2B Online Remoto (Waveforms e Metadados Compartilhados via Rede):**
  - Implementado `RemoteTrackMessage` e variantes `B2bMessage::TrackLoaded` e `B2bMessage::TrackUnloaded`.
  - Ao carregar uma faixa no deck próprio, o openDJ extrai os metadados (Título, Artista, BPM, Tom, Duração, Primeiro Beat) e os dados de waveform (nível 3 da pirâmide em ~125 bins/s e o resumo de 256 fatias de overview).
  - O DJ remoto reconstrói a `Track` com `Waveform::from_finest` e insere no `LoadedTracks` do engine, permitindo que a GPU renderize tanto o overview completo quanto a forma de onda contínua rolante (*scrolling waveform*) sem necessidade do arquivo de áudio físico estar no disco.
  - Buffer de rede expandido para 1 MB em `live_b2b.rs` para acomodar pacotes de waveform e frames de vídeo JPEG.
- **B2B Online Remoto (Auto-Descoberta UDP na Rede Local):**
  - Implementado sistema de auto-descoberta via UDP broadcast na porta `7499` (`live_b2b::B2bBroadcaster` e `live_b2b::B2bDiscoveryHandle`): o anfitrião anuncia seu código de sala e o convidado localiza o IP do host automaticamente sem precisar digitá-lo manualmente.
  - Estado 4 adicionado na UI do Slint ("Procurando na rede…") com timeout de 15 segundos e cancelamento a qualquer momento.
  - Fallback automático: caso a rede bloqueie UDP broadcast ou não encontre o anfitrião, a interface exibe o campo manual de IP para conexão direta (`b2b-join-direct`).
- **B2B Online Remoto (Suporte a Câmera Remota no Compositor Vertical):**
  - Implementado `Session::set_remote_camera` em `crates/live/src/session.rs` para permitir alimentação direta do slot de câmera 1 (DJ B / convidado) a partir da rede.
  - Adicionado `B2bMessage::CameraFrame { width, height, jpeg }` em `live_b2b.rs` com buffer expandido (512 KB) para transmissão contínua de frames comprimidos da webcam do parceiro.
  - Decodificação de imagem em `live.rs` integrada à cena vertical 9:16 do TikTok (as câmeras do DJ 1 e DJ 2 aparecem lado a lado no topo da transmissão).
- **B2B Online Remoto (Sincronização de Rede e Espelhamento em Tempo Real):**
  - Implementado motor de rede `live_b2b::B2bNetworkSession` com suporte nativo a Host TCP e Convidado TCP de baixa latência (porta `7498` por padrão, TCP_NODELAY ativado).
  - Protocolo de mensagens `B2bMessage` e `DeckStateMessage` codificado em JSON compacto (~40 bytes por frame): transmite Play/Pause, posição da agulha (`playposition`), BPM, volume, pitch slider (`rate`), equalizadores (`eq_low`, `eq_mid`, `eq_high`) e medidor de VU (`vu_meter`).
  - Sincronização automática contínua em 30 Hz integrada ao loop de polling do LIVE: o deck do anfitrião envia seu estado e espelha em tempo real os faders, agulha e ondas do DJ convidado, e vice-versa.
- **B2B Online Remoto (Modo Colaborativo sem Trava Engessada):**
  - Ambos os decks (controles de transporte, EQs, graves, médios, agudos, faders e jog/pitch bend) permanecem **100% liberados** para ambos os DJs, permitindo transições completas e ajustes finos de beatmatching durante o set/batalha.
  - Indicador visual em cada deck (`deck.slint`): exibe uma pílula informativa no topo do deck identificando a posse de cada faixa ("B2B · <Nome do DJ>").
- **B2B Online Remoto (Fase 1 - Interface e Código de Sala):**
  - Seção "B2B ONLINE" adicionada ao painel LIVE com indicador de status em tempo real.
  - Botões para "Criar Sala" (modo anfitrião) e "Entrar com Código" (modo convidado).
  - Geração de código numérico de 4 dígitos (1000–9999) via `live_b2b::generate_room_code`.
  - Botão "Copiar" que transfere o código diretamente para a área de transferência do macOS via `pbcopy`.
  - Seletor de posse de deck ("MEU DECK: Deck A / Deck B") para definir qual deck cada DJ opera.
  - Campo de entrada com botão "Conectar" para o DJ convidado inserir o código de 4 dígitos.
  - Opções para cancelar ou encerrar a sessão B2B a qualquer momento.
- **Janela de Projetor TikTok LIVE Studio:**
  - Janela nativa 9:16 (`LiveProjectorWindow`) com suporte a tela cheia e captura direta de janela sem drivers virtuais.

## 2026-11-18 (tarde)

### Adicionado
- **Battle Live: live vertical no TikTok, sem OBS.** Botão **LIVE** na barra
  superior (fora do modo Player). O painel tem:
  - modo **Solo · 1 DJ** ou **B2B · 2 DJs** e o nome de cada DJ;
  - um QR code para cada celular. O celular abre uma página do próprio
    openDJ (HTTPS na rede Wi-Fi) e manda a câmera, sem instalar aplicativo.
    Na primeira vez o celular avisa que a conexão "não é particular", porque
    o certificado foi feito pelo próprio notebook: é só seguir para o site;
  - a prévia do que o público vê (720 × 1280);
  - troca de câmera a cada 8, 16 ou 32 compassos e escolha de cena na mão;
  - URL do servidor e chave de transmissão do TikTok (a chave fica nas
    Chaves do macOS, não no arquivo de configurações) e gravação em MP4
    na pasta das gravações.
- **Diretor de cenas** que segue o que está soando, não o crossfader: o
  nível de cada canal com o fader e o crossfader aplicados, acima de −30 dB.
  A faixa com menos tempo restante é a que sai; o outro DJ é o próximo.
  Testado com 12 casos (fader do canal baixo, mixagem pelos dois, faixa
  acabando, troca a cada 16 compassos, mínimo de 4 compassos por cena, cena
  na mão, Solo com uma ou duas câmeras).
- **Segunda cópia do master no motor** (`take_live_feed`), para a live
  poder rodar junto com a rádio pela internet.
- Arquivos: novo crate `crates/live` (`phones`, `compose`, `director`,
  `output`, `session`, página `camera.html`), `crates/app/src/live.rs`,
  `crates/app/ui/live.slint`, `crates/engine/src/engine.rs`,
  `crates/engine/src/lib.rs`; plano em
  `openspec/changes/add-battle-live/`.

### Verificado na nuvem
- 26 testes do crate `opendj-live`, incluindo um celular falso mandando
  imagem pelo HTTPS e uma gravação real com o ffmpeg: 3,02 s de imagem para
  2,99 s de som (imagem e som juntos).

### Falta testar no Mac
- Celulares de verdade (iPhone e Android), o custo de copiar a tela do
  openDJ no MacBook Air e a transmissão no TikTok com a chave.
- A votação #A/#B no chat ainda não conta votos: o TikTok não tem uma forma
  oficial de um aplicativo ler o chat. A barra existe no desenho e aparece
  quando houver votos.

## 2026-11-18

### Verificado
- Branch `smaykdj` compilado no MacBook Air (release, 11m52s) sem erros,
  incluindo o MAP MIDI completo e o progresso dos stems.
- Para abrir: `cargo run --release -p opendj --bin opendj` (o projeto tem mais de um
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
