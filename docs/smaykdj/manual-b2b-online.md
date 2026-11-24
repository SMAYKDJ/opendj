# Manual do B2B Online — openDJ edição SMAYKDJ

> Dois DJs. Dois computadores. Uma live só.

Este manual cobre tudo o que você precisa saber para usar o modo **B2B Online** do openDJ: desde a primeira conexão na mesma rede Wi-Fi até uma live pela internet com transmissão no TikTok LIVE Studio.

---

## Índice

1. [O que é o B2B Online](#1-o-que-é-o-b2b-online)
2. [Requisitos](#2-requisitos)
3. [Passo a Passo — DJ Anfitrião (Host)](#3-passo-a-passo--dj-anfitrião-host)
4. [Passo a Passo — DJ Convidado (Guest)](#4-passo-a-passo--dj-convidado-guest)
5. [Conexão pela Internet (fora da LAN)](#5-conexão-pela-internet-fora-da-lan)
6. [Waveforms e Metadados Compartilhados](#6-waveforms-e-metadados-compartilhados)
7. [Áudio do Parceiro (Opus)](#7-áudio-do-parceiro-opus)
8. [Câmera Remota](#8-câmera-remota)
9. [Modo Colaborativo (sem Deck Lock)](#9-modo-colaborativo-sem-deck-lock)
10. [Transmissão com TikTok LIVE Studio](#10-transmissão-com-tiktok-live-studio)
11. [Troubleshooting / Perguntas Frequentes](#11-troubleshooting--perguntas-frequentes)
12. [Glossário](#12-glossário)
13. [Referência Técnica](#13-referência-técnica)

---

## 1. O que é o B2B Online

O B2B Online permite que **dois DJs**, cada um no seu computador e potencialmente em locais diferentes, dividam a mesma sessão do openDJ de forma totalmente sincronizada:

- Cada DJ vê as **waveforms dos dois decks** (a sua e a do parceiro) correndo lado a lado.
- O **áudio do parceiro** é transmitido em tempo real via codec Opus e misturado no master, pronto para a live e a gravação.
- A **câmera do parceiro** alimenta a cena vertical 9:16 do compositor do TikTok.
- Todos os **controles** (Play, Pause, Cue, Faders, EQs, Pitch, Jog) são sincronizados em **30 Hz**.
- Não precisa de conta, servidor externo nem aplicativo: a conexão é direta entre os dois openDJs.

---

## 2. Requisitos

### Requisitos mínimos

| Item | Detalhe |
|------|---------|
| **openDJ** | Edição SMAYKDJ, branch `smaykdj`, compilado com `cargo build --release -p opendj` |
| **macOS** | Apple Silicon (M1 ou superior recomendado) |
| **Rust** | Instalado (`rustup`, `cargo`) |
| **ffmpeg** | `brew install ffmpeg` (necessário para a live no TikTok) |
| **Rede** | Wi-Fi na mesma rede local (**LAN**) ou conexão pela internet com porta liberada |

### Portas utilizadas

| Porta | Protocolo | Uso |
|-------|-----------|-----|
| `7498` | TCP | Canal de dados principal (controles, waveforms, áudio Opus, câmera) |
| `7499` | UDP | Auto-descoberta do anfitrião na rede local (broadcast) |
| `19302` / `3478` | UDP | Consulta STUN para descobrir IP público (Google / Cloudflare) |

### Para conexão pela internet

- O anfitrião precisa **liberar a porta 7498 TCP** no roteador (port forwarding) **ou** usar uma VPN (Tailscale, Hamachi, ZeroTier).
- O STUN descobre o IP público automaticamente, mas **não perfura NAT por conta própria** — a porta precisa estar acessível.

---

## 3. Passo a Passo — DJ Anfitrião (Host)

### 3.1. Abrir o painel LIVE

1. Clique no botão **LIVE** na barra superior do openDJ (fora do modo Player).
2. No painel, localize a seção **B2B ONLINE**.

### 3.2. Criar a sala

1. Clique em **Criar Sala B2B**.
2. O openDJ gera um **código de 4 dígitos** (ex: `7842`) e abre uma conexão TCP na porta `7498`.
3. Em segundo plano, o openDJ:
   - Inicia o **broadcaster UDP** (porta `7499`) anunciando o código da sala na rede local.
   - Consulta servidores **STUN** para descobrir o IP público (caso queira jogar pela internet).

### 3.3. Compartilhar o código

- Clique em **Copiar** para copiar o código para a área de transferência.
- Se o STUN já respondeu, o texto copiado inclui o IP público:
  ```
  Código: 7842 (IP internet: 201.86.12.34:7498)
  ```
- Envie esse texto pelo WhatsApp, Discord ou qualquer chat para o DJ convidado.

### 3.4. Escolher o deck

- No seletor **MEU DECK**, escolha se você opera o **Deck A** ou o **Deck B**.
- O parceiro ficará com o outro deck. (Ambos podem operar ambos — veja [Modo Colaborativo](#9-modo-colaborativo-sem-deck-lock).)

### 3.5. Esperar a conexão

- Quando o convidado se conectar, o indicador de status muda para **Conectado** e o nome do DJ parceiro aparece na pílula sobre o deck dele.
- Pronto para tocar!

---

## 4. Passo a Passo — DJ Convidado (Guest)

### 4.1. Abrir o painel LIVE

1. Clique em **LIVE** na barra superior.
2. Na seção **B2B ONLINE**, clique em **Entrar com Código**.

### 4.2. Digitar o código

1. No campo de entrada, digite o **código de 4 dígitos** que o anfitrião compartilhou (ex: `7842`).
2. Clique em **Conectar**.

### 4.3. Auto-descoberta na rede local

- O openDJ inicia uma **busca UDP na rede local** (porta `7499`).
- O status muda para **"Procurando na rede…"**.
- Se encontrar o anfitrião, conecta automaticamente em poucos segundos — **sem precisar digitar IP**.
- Se não encontrar em **15 segundos**, o openDJ oferece o campo **IP manual** como fallback.

### 4.4. Conexão direta por IP (se necessário)

Se a auto-descoberta não funcionou (ex: roteador bloqueia broadcast UDP), ou se o anfitrião está na **internet**:

1. No campo que aparece, digite o IP ou IP:Porta:
   - LAN: `192.168.1.42` (a porta `7498` é assumida)
   - Internet: `201.86.12.34:7498`
2. Clique em **Conectar**.

### 4.5. Pronto

- O status muda para **Conectado**.
- Os controles, waveforms e áudio do anfitrião começam a fluir em tempo real.

---

## 5. Conexão pela Internet (fora da LAN)

### 5.1. STUN e IP público

Quando o anfitrião cria a sala, o openDJ consulta automaticamente servidores STUN (Google: `stun.l.google.com:19302`, Cloudflare: `stun.cloudflare.com:3478`) para descobrir o IP público e a porta mapeada pelo NAT.

O IP aparece na UI e é incluído no texto do botão **Copiar**.

### 5.2. Port Forwarding no roteador

Para que o convidado consiga alcançar o anfitrião pela internet:

1. Acesse o painel do seu roteador (geralmente `192.168.1.1`).
2. Crie uma regra de **Port Forwarding**:
   - **Porta externa:** `7498`
   - **Porta interna:** `7498`
   - **Protocolo:** TCP
   - **IP interno:** o IP local do seu Mac (ex: `192.168.1.42`)
3. Salve e teste.

### 5.3. Alternativa: VPN

Se não quiser mexer no roteador:

- **Tailscale** (recomendado): instale nos dois Macs, entre na mesma rede Tailscale. Use o IP do Tailscale (ex: `100.64.x.x`) no campo de conexão do convidado.
- **Hamachi** ou **ZeroTier**: funcionam da mesma forma.

### 5.4. No convidado

O convidado digita diretamente o IP público (ou IP da VPN) no campo de conexão. O openDJ detecta que é um IP (contém `.` ou `:`) e conecta diretamente, sem esperar a descoberta UDP.

---

## 6. Waveforms e Metadados Compartilhados

### Como funciona

Quando qualquer DJ carrega uma faixa no seu deck:

1. O openDJ extrai os **metadados** (Título, Artista, BPM, Tom, Duração, posição do primeiro beat).
2. Extrai os dados de **waveform** em dois níveis:
   - **Nível 3** (~125 bins por segundo): a forma de onda detalhada para o *scrolling waveform*.
   - **Overview** (256 fatias): o resumo que aparece na barra superior do deck.
3. Envia tudo pela rede via `B2bMessage::TrackLoaded` (~30 KB por faixa).

### No lado do parceiro

O openDJ do parceiro reconstrói uma `Track` completa a partir dos dados recebidos usando `Waveform::from_finest`, e insere no motor de renderização (`LoadedTracks`). A GPU renderiza as ondas normalmente — **não é preciso ter o arquivo de áudio no disco do parceiro**.

### Quando a faixa é descarregada

Uma mensagem `TrackUnloaded` limpa o deck correspondente na tela do parceiro.

---

## 7. Áudio do Parceiro (Opus)

### Transmissão automática

O áudio é transmitido automaticamente assim que a conexão B2B é estabelecida. Não há botão para ligar ou desligar.

### Especificações

| Parâmetro | Valor |
|-----------|-------|
| **Codec** | Opus (`opus = "0.4"`) |
| **Bitrate** | 128 kbps estéreo |
| **Tamanho do frame** | 960 amostras × 2 canais = 20 ms @ 48 kHz |
| **Latência** | ~30–50 ms (rede local) |
| **Aplicação** | Low Delay (`OPUS_APPLICATION_RESTRICTED_LOWDELAY`) |

### Fluxo

1. O encoder drena o áudio do **master feed** local em tempo real.
2. Comprime blocos de 20 ms (960 × 2 amostras float32) em pacotes Opus.
3. Envia via TCP como `B2bMessage::AudioOpus { data }`.
4. O decoder do parceiro descomprime e insere no buffer `remote_audio`.
5. O `Session::push_remote_audio()` mistura o áudio remoto no master — aparecendo tanto na **live do TikTok** quanto na **gravação MP4**.

### Sem eco

Como cada DJ transmite apenas o áudio do **seu próprio deck**, não há risco de eco ou loop de feedback.

---

## 8. Câmera Remota

### Funcionamento

Cada DJ aponta um celular (câmera via WebSocket JPEG, como na live Solo) para si. O openDJ do anfitrião envia os frames JPEG da câmera local pela rede como `B2bMessage::CameraFrame`.

No openDJ do parceiro, `Session::set_remote_camera()` alimenta o **slot 1** (DJ B) do compositor vertical 9:16, posicionando as duas câmeras lado a lado no topo da transmissão.

### Qualidade

- Resolução: até 960 px no lado maior
- Taxa: até 24 FPS
- Formato: JPEG comprimido
- Buffer de rede: 1 MB (suficiente para frames JPEG e waveforms)

---

## 9. Modo Colaborativo (sem Deck Lock)

### Filosofia

O openDJ **não trava** nenhum deck. Ambos os DJs têm **100% de controle** sobre ambos os decks a qualquer momento:

- Play / Pause / Cue
- Volume e Crossfader
- EQ (Low, Mid, High)
- Pitch Bend e Jog Wheel
- Carregamento de faixas

### Badge visual de posse

Para saber quem carregou qual faixa, cada deck exibe uma **pílula informativa** no topo:

```
╔══════════════════════════════╗
║  B2B · DJ SMAYK             ║
╚══════════════════════════════╝
```

Essa pílula indica quem é o "dono" daquela faixa — ou seja, quem a carregou — mas **não impede** o outro DJ de mexer nos controles.

### Por quê?

O bom DJ deve se atentar nos seus ofícios. Se errar, menos pontos com o público — e é exatamente assim que uma batalha de verdade funciona.

---

## 10. Transmissão com TikTok LIVE Studio

### Passo a passo

1. No openDJ do **anfitrião**, clique em **Abrir janela (TikTok Studio)** no painel LIVE.
2. Uma janela dedicada vertical 9:16 chamada **"openDJ - TikTok Live"** se abrirá a 30 FPS.
3. Abra o **TikTok LIVE Studio** no mesmo Mac.
4. Clique em **Adicionar Fonte › Captura de Janela (Window Capture)**.
5. Selecione a janela **"openDJ - TikTok Live"**.
6. A cena composta (câmeras no topo + decks embaixo) já está pronta para transmitir.

### Layout da cena vertical (720×1280)

```
┌────────────────────────┐
│  TikTok UI (≈7%)       │  ← conta, LIVE, espectadores
├────────────────────────┤
│  📷 DJ 1  │  📷 DJ 2  │  ← câmeras lado a lado
├────────────────────────┤
│  PRÓXIMO: DJ 2 · 0:42  │  ← aviso de troca (B2B)
├────────────────────────┤
│                        │
│  Tela do openDJ        │  ← decks, waveforms, mixer
│                        │
├────────────────────────┤
│  Comentários do TikTok │  ← zona segura (≈30%)
└────────────────────────┘
```

### Dicas

- O **diretor de cenas** alterna automaticamente entre as câmeras seguindo quem está tocando (não o crossfader).
- A troca acontece a cada intervalo configurável (8, 16 ou 32 compassos).
- Tempo mínimo por cena: 4 compassos, para não piscar.
- Tecla ou pad MIDI pode forçar a cena manualmente.

---

## 11. Troubleshooting / Perguntas Frequentes

### A auto-descoberta não encontra o anfitrião

**Causa provável:** o roteador bloqueia broadcast UDP na porta `7499` (comum em redes corporativas ou com isolamento de clientes ativado).

**Solução:** depois do timeout de 15 s, use o campo **IP manual** e digite o IP local do anfitrião (ex: `192.168.1.42`).

---

### O convidado não consegue conectar (timeout)

**Verificar:**

1. Os dois Macs estão na mesma rede Wi-Fi?
2. O Firewall do macOS está bloqueando? Vá em **Ajustes do Sistema › Rede › Firewall** e permita conexões de entrada para o openDJ.
3. Se pela internet: a porta `7498` TCP foi liberada no roteador do anfitrião?

---

### A waveform do parceiro não aparece

**Causa:** a faixa foi carregada antes da conexão ser estabelecida.

**Solução:** descarregue e recarregue a faixa, ou o parceiro carrega uma nova faixa. A waveform é enviada no momento do carregamento.

---

### O áudio do parceiro não chega

**Verificar:**

1. A conexão está ativa? (indicador "Conectado" no painel)
2. O volume do master no openDJ do parceiro está acima de zero?
3. Se tudo parece certo, desconecte e reconecte — o encoder/decoder Opus é reiniciado a cada nova conexão.

---

### O STUN não encontra o IP público

**Causas possíveis:**

- Sem acesso à internet.
- Firewall bloqueando UDP para as portas `19302` / `3478`.
- NAT simétrico (symmetric NAT) — nesse caso, use Tailscale/VPN.

**O IP público não é obrigatório** — se ambos estão na LAN, a auto-descoberta UDP funciona sem ele.

---

### Latência alta / áudio cortando

- Use **cabo Ethernet** em vez de Wi-Fi quando possível.
- Feche programas que consomem muita banda (downloads, streaming).
- Se pela internet, a latência depende da rota entre os dois ISPs — um relay/VPN com servidor próximo pode ajudar.

---

### Como saber se estou conectado?

O painel LIVE mostra o status em tempo real:

| Status | Significado |
|--------|-------------|
| **Desconectado** | Nenhuma sala ativa |
| **Aguardando…** | Sala criada, esperando o convidado |
| **Procurando na rede…** | Convidado buscando via UDP |
| **Conectado** | Tudo funcionando! |

---

## 12. Glossário

| Termo | Significado |
|-------|-------------|
| **B2B** | *Back to Back* — dois DJs tocando alternadamente na mesma sessão |
| **Host / Anfitrião** | O DJ que cria a sala e espera a conexão |
| **Guest / Convidado** | O DJ que entra na sala com o código |
| **Opus** | Codec de áudio de baixa latência usado para transmitir o som do parceiro |
| **STUN** | Protocolo para descobrir o IP público atrás de um NAT |
| **NAT** | *Network Address Translation* — o mecanismo que esconde seu IP real atrás do IP do roteador |
| **Port Forwarding** | Regra no roteador que redireciona uma porta externa para o seu Mac |
| **UDP Broadcast** | Mensagem enviada para todos os dispositivos da rede local de uma vez |
| **Waveform** | A representação visual da forma de onda do áudio |
| **Overview** | O resumo compacto da waveform (256 fatias) que aparece na barra do deck |
| **Master Feed** | O canal de áudio principal que combina todos os decks |
| **Compositor 9:16** | O módulo que monta a cena vertical (câmeras + decks) para o TikTok |

---

## 13. Referência Técnica

### Protocolo de rede

- **Transporte:** TCP direto com `TCP_NODELAY` para mínima latência.
- **Formato:** mensagens JSON prefixadas por 4 bytes little-endian (tamanho do payload).
- **Buffer máximo:** 1 MB por mensagem.

### Tipos de mensagem (`B2bMessage`)

| Variante | Direção | Frequência | Descrição |
|----------|---------|------------|-----------|
| `DeckState` | Bidirecional | 30 Hz | Play, posição da agulha, BPM, faders, EQs, VU meter |
| `TrackLoaded` | Bidirecional | Por evento | Metadados + waveform (~30 KB) ao carregar faixa |
| `TrackUnloaded` | Bidirecional | Por evento | Limpa o deck do parceiro |
| `AudioOpus` | Bidirecional | ~50/s (20ms) | Pacote de áudio Opus comprimido |
| `CameraFrame` | Bidirecional | ≤24/s | Frame JPEG da câmera do celular |

### Arquivos-chave no código

| Arquivo | Responsabilidade |
|---------|------------------|
| `crates/app/src/live_b2b.rs` | B2bMessage, B2bNetworkSession, B2bBroadcaster, B2bDiscoveryHandle, STUN, Opus helpers |
| `crates/app/src/live.rs` | Live struct, callbacks, loop de polling 30 Hz, integração dos módulos |
| `crates/app/ui/live.slint` | Interface do painel LIVE (B2B Online, campos, botões, status) |
| `crates/live/src/session.rs` | Session, push_remote_audio(), mixagem no master, remote_audio buffer |

### Diagrama de arquitetura

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            REDE (TCP :7498)                                 │
│                                                                              │
│  ┌──────────────────────────┐          ┌──────────────────────────┐          │
│  │   openDJ — DJ 1 (Host)  │◄────────►│  openDJ — DJ 2 (Guest)  │          │
│  ├──────────────────────────┤          ├──────────────────────────┤          │
│  │ Deck A (Controle Local)  │          │ Deck A (Visualiza DJ 1) │          │
│  │ Deck B (Visualiza DJ 2) │          │ Deck B (Controle Local)  │          │
│  ├──────────────────────────┤          ├──────────────────────────┤          │
│  │ Encoder Opus ──────────────────────► Decoder Opus ─► Master   │          │
│  │ Master ◄──────────────────────────── Encoder Opus             │          │
│  ├──────────────────────────┤          ├──────────────────────────┤          │
│  │ Câmera (cel.) ─────────────────────► Slot 1 Compositor        │          │
│  │ Slot 1 Compositor ◄────────────────── Câmera (cel.)           │          │
│  └────────┬─────────────────┘          └─────────────────────────┘          │
│           │                                                                  │
│           ▼                                                                  │
│  ┌──────────────────┐                                                        │
│  │ Compositor 9:16  │                                                        │
│  │  720 × 1280      │                                                        │
│  └────────┬─────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│  ┌──────────────────────────────┐                                            │
│  │ Janela "openDJ - TikTok Live"│                                           │
│  │ ► Captura no TikTok Studio   │                                           │
│  │ ► Ou RTMP direto             │                                           │
│  └──────────────────────────────┘                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

> **Versão:** SMAYKDJ branch `smaykdj` · Novembro 2026
>
> **Veja também:** [b2b-online.md](b2b-online.md) (roadmap técnico) · [live-tiktok.md](live-tiktok.md) (Battle Live solo) · [CHANGELOG.md](CHANGELOG.md)
