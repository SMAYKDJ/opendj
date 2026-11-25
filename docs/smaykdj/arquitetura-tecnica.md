# Arquitetura Técnica — openDJ edição SMAYKDJ

> Referência completa para desenvolvedores e manutenção do projeto.

---

## Índice

1. [Visão Geral](#1-visão-geral)
2. [Crates e Módulos](#2-crates-e-módulos)
3. [Pipeline de Live (Battle Live)](#3-pipeline-de-live-battle-live)
4. [Sistema de Câmeras](#4-sistema-de-câmeras)
5. [Compositor Visual (tiny-skia)](#5-compositor-visual-tiny-skia)
6. [Diretor de Cenas](#6-diretor-de-cenas)
7. [Codificação e Transmissão (output.rs)](#7-codificação-e-transmissão-outputrs)
8. [B2B Online](#8-b2b-online)
9. [Sistema de Licenciamento](#9-sistema-de-licenciamento)
10. [Segurança e Anti-Tamper](#10-segurança-e-anti-tamper)
11. [Configurações Persistentes](#11-configurações-persistentes)
12. [Perfis de Compilação](#12-perfis-de-compilação)
13. [Diagrama de Dependências](#13-diagrama-de-dependências)

---

## 1. Visão Geral

```
┌─────────────────────────────────────────────────────────────────┐
│                     openDJ (crates/app)                         │
│  Slint UI ←→ live.rs ←→ opendj-live (crates/live)              │
│              ↕              ↕                                    │
│           license.rs    security.rs                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
     ┌────────▼────────┐    ┌──────────▼─────────┐
     │  crates/live    │    │   crates/engine     │
     │  phones.rs      │    │   engine.rs         │
     │  compose.rs     │    │   (audio engine,    │
     │  director.rs    │    │    stems, library)  │
     │  session.rs     │    └────────────────────┘
     │  output.rs      │
     │  live_b2b.rs    │
     └─────────────────┘
```

---

## 2. Crates e Módulos

### `crates/app` — Aplicação principal

| Arquivo | Responsabilidade |
|---------|-----------------|
| `src/main.rs` | Ponto de entrada, inicialização do Slint |
| `src/live.rs` | Integração UI ↔ opendj-live, callbacks, settings |
| `src/live_b2b.rs` | B2B Online: sala, STUN, Opus, sincronização |
| `src/license.rs` | Validação de licença Ed25519, hardware ID |
| `src/security.rs` | Anti-debug, ObfStr, PT_DENY_ATTACH |
| `ui/live.slint` | Interface do painel Battle Live (Slint DSL) |

### `crates/live` — Núcleo de transmissão

| Arquivo | Responsabilidade |
|---------|-----------------|
| `src/phones.rs` | Servidor HTTPS para câmeras dos celulares |
| `src/compose.rs` | Compositor 720×1280 com tiny-skia |
| `src/director.rs` | Lógica de troca de cena (automática/manual) |
| `src/session.rs` | Orquestra compositor + diretor + output |
| `src/output.rs` | FFmpeg subprocess: H.264 VideoToolbox + AAC + RTMP |
| `src/lib.rs` | Tipos públicos (`Frame`, `Scene`, `Shot`) |
| `src/camera.html` | Página web servida para os celulares |

---

## 3. Pipeline de Live (Battle Live)

```
Celular 1 ──HTTPS/WebSocket──► phones.rs ──► Frame{rgba, w, h}
Celular 2 ──HTTPS/WebSocket──► phones.rs ──► Frame{rgba, w, h}
                                                      │
                                                      ▼
Tela do Mac ──────────────────────────────► compose.rs::set_screen()
                                                      │
                                                      ▼
                                           compose.rs::render()
                                           ↑ director.rs::Shot
                                                      │
                                                 RGBA 720×1280
                                                      │
                                                      ▼
Áudio Master ──────────────────────────────► output.rs (TCP port)
                                                      │
                                                      ▼
                                             ffmpeg subprocess
                                             ├─ H.264 VideoToolbox
                                             └─ AAC 160 kbps
                                                      │
                                        ┌─────────────┴──────────┐
                                        ▼                        ▼
                               rtmp://push.tiktok.com    ~/Music/openDJ/
                               /live/CHAVE              Recordings/*.mp4
```

---

## 4. Sistema de Câmeras

### Como funciona (`crates/live/src/phones.rs`)

O openDJ sobe um **servidor HTTPS local** (certificado autoassinado gerado
no momento). Os celulares acessam via QR code e abrem `camera.html`.

A página usa a **API MediaDevices** do navegador para capturar a câmera e
envia frames como JPEG via **WebSocket** (também sobre HTTPS/WSS).

```
Mac (openDJ)                     Celular (Chrome/Safari)
     │                                    │
     │── QR code (URL wss://192.168.x.x) ──►│
     │                                    │
     │◄── WebSocket (JPEG frames ~15fps) ──│
     │                                    │
phones.rs::decode_jpeg()                  │
     │                                    │
Frame{rgba} → Composer::set_camera(slot)  │
```

### Chave de transmissão no Keychain

A chave RTMP **nunca é salva em disco** como texto. Ela é guardada no
**Keychain do macOS** (`security add-generic-password`), acessível apenas
pelo seu usuário. Ao abrir o openDJ, a chave é lida do Keychain e mantida
só na memória.

---

## 5. Compositor Visual (tiny-skia)

**Arquivo:** `crates/live/src/compose.rs`

Usa `tiny-skia` (renderer 2D puro Rust) para desenhar cada frame 720×1280:

```
┌──────────────────────────────────────────────────────┐
│ y=0..65   faixa reservada TikTok (topo)              │
├──────────────────────────────────────────────────────┤
│ y=65..130 barra de próximo DJ / tempo restante       │
├──────────────────────────────────────────────────────┤
│ y=130..355 CÂMERAS (10px margem, 8px gap)           │
│            Scene::Both   → lado a lado, igual        │
│            Scene::Only(n)→ câmera n em tela cheia    │
│            Scene::Solo{main} → ambas, lado a lado    │
├──────────────────────────────────────────────────────┤
│ y=355..695 TELA openDJ (shrink se Retina > ~1080px) │
├──────────────────────────────────────────────────────┤
│ y=695..730 barra de votos A|B (placar TikTok)        │
├──────────────────────────────────────────────────────┤
│ y=730..1215 área livre / comentários TikTok          │
├──────────────────────────────────────────────────────┤
│ y=1215..1280 rodapé "openDJ  B2B BATTLE"             │
└──────────────────────────────────────────────────────┘
```

**Coordenadas no sistema interno (design 540 × 960):**
- `scale = width / 540.0` → tudo escalado automaticamente
- Câmeras: `(10, 76, 520, 224)` em unidades de design

---

## 6. Diretor de Cenas

**Arquivo:** `crates/live/src/director.rs`

```rust
pub enum Scene {
    Both,           // B2B: ambos os DJs lado a lado
    Only(usize),    // apenas o DJ n (0 = A, 1 = B)
    Solo { main: usize }, // Solo: ambas câmeras, mesmo tamanho
}
```

**Regras de troca automática:**
- Mínimo de **4 compassos** por cena (evita flicker)
- Padrão: troca a cada **16 compassos** (configurável: 8, 16, 32)
- Em B2B: troca entre `Both` ↔ `Only(DJ com mais nível de áudio)`
- Em Solo: alterna entre câmeras mas sempre `Scene::Solo`
- Se só há 1 câmera conectada: `Scene::Only(0)` fixo

---

## 7. Codificação e Transmissão (`output.rs`)

### Fluxo do ffmpeg

```bash
# Vídeo: pipe RGBA do compositor
ffmpeg \
  -f rawvideo -pix_fmt rgba -s 720x1280 -r 30 -i pipe:0 \
  # Áudio: TCP socket interno (f32le stereo 48kHz)
  -f f32le -ar 48000 -ac 2 -i tcp://127.0.0.1:{PORT} \
  # Encoder vídeo (macOS: VideoToolbox GPU)
  -c:v h264_videotoolbox -realtime 1 -allow_sw 1 \
  -b:v 3000k -maxrate 3000k -bufsize 6000k \
  -g 60 -pix_fmt yuv420p \
  # Encoder áudio
  -c:a aac -b:a 160k \
  # Saída: RTMP e/ou arquivo
  -f flv rtmp://push.tiktok.com/live/CHAVE
```

### VideoToolbox vs x264

| Situação | Encoder |
|----------|---------|
| macOS (qualquer) | `h264_videotoolbox` (GPU, baixo consumo de CPU) |
| Linux / Windows | `libx264 -preset veryfast -tune zerolatency` |

### Muxer `tee` (gravar + transmitir)

Quando os dois switches estão ligados:
```
-f tee "[f=flv]rtmp://push.tiktok.com/live/CHAVE|[f=mp4]/path/live.mp4"
```

---

## 8. B2B Online

### Protocolos

| Protocolo | Porta | Uso |
|-----------|-------|-----|
| TCP | 7498 | Dados B2B (waveform, metadados, controle) |
| UDP | 7499 | Descoberta automática na LAN (broadcast) |
| STUN (UDP) | 19302 / 3478 | Descoberta do IP público (internet) |
| TCP | auto | Áudio Opus (stream contínuo) |

### Fluxo de conexão

```
DJ A (host)                          DJ B (guest)
    │                                     │
    │── broadcast UDP 7499 ──────────────►│
    │◄── resposta UDP ────────────────────│
    │                                     │
    │── TCP 7498 (aceitar conexão) ◄──────│
    │                                     │
    │◄──► B2bMessage (JSON) ─────────────►│
    │      TrackLoaded / TrackUnloaded    │
    │      AudioOpus (frames 20ms)        │
    │      Heartbeat                      │
```

### Mensagens B2B

```rust
pub enum B2bMessage {
    Hello { name: String, color: u32 },
    TrackLoaded(RemoteTrackMessage),  // waveform + metadados
    TrackUnloaded { deck: usize },
    AudioOpus { data: Vec<u8> },      // frame Opus 20ms @ 128kbps
    Heartbeat,
}
```

---

## 9. Sistema de Licenciamento

**Arquivo:** `crates/app/src/license.rs`

### Formato da licença

```
ODJ1.<PAYLOAD_BASE64>.<ASSINATURA_BASE64>
```

**Payload (JSON):**
```json
{
  "machine_id": "MACH-XXXX-XXXX-XXXX-XXXX",
  "issued_at": 1234567890,
  "expires_at": null,
  "tier": "pro"
}
```

### Hardware ID

```
MACH-{SHA256(IOPlatformSerialNumber)[0..4]}-...-...
```

Gerado a partir do serial do hardware via `IOKit` (macOS). Único por máquina.

### Criptografia

- Algoritmo: **Ed25519** (via crate `ring`)
- Par de chaves: `--keygen` gera `private_key.b64` + `public_key.b64`
- A chave pública fica **embutida** no binário em tempo de compilação
- A chave privada fica **somente** com o vendedor (você)

### Estado atual

```rust
// crates/app/src/license.rs
pub const ENFORCE_LICENSE: bool = false; // ← mude para true ao comercializar
```

Com `false`: todas as funções liberadas, UI mostra "TESTES LIBERADOS".

### Gerar licença para um cliente

```bash
# 1. Gerar par de chaves (só uma vez)
./target/release/opendj --keygen

# 2. Pegar o hardware ID do cliente
./target/release/opendj --machine-id

# 3. Gerar a licença
./target/release/opendj --create-license MACH-XXXX-XXXX-XXXX-XXXX
# Output: ODJ1.eyJ...}.sig=...
```

---

## 10. Segurança e Anti-Tamper

**Arquivo:** `crates/app/src/security.rs`

| Proteção | Implementação |
|----------|--------------|
| Anti-debug macOS | `PT_DENY_ATTACH` via syscall |
| Detecção de Frida/Cycript | Verifica `DYLD_INSERT_LIBRARIES` |
| Detecção de processo pai | Verifica se pai é `lldb`, `gdb` etc. |
| Strings ofuscadas | `ObfStr` — decodifica apenas em runtime |
| Strip de símbolos | `strip = true` no perfil `commercial` |
| LTO fat | `lto = "fat"` — impede análise de IR intermediário |

---

## 11. Configurações Persistentes

**Módulo:** `settings` (em `crates/app/src/`)

| Chave | Valor padrão | Descrição |
|-------|-------------|-----------|
| `live_mode_b2b` | `true` | B2B ou Solo |
| `live_dj_a` | `"DJ A"` | Nome do DJ A |
| `live_dj_b` | `"DJ B"` | Nome do DJ B |
| `live_rotate_bars` | `16` | Compassos por cena |
| `live_stream` | `true` | Transmitir ativado |
| `live_record` | `true` | Gravar ativado |
| `live_rtmp_server` | `"rtmp://push.tiktok.com/live/"` | URL servidor |
| `rtmp_key` | *(Keychain)* | Chave de transmissão |

**Localização:** `~/Library/Application Support/openDJ/settings.json`

---

## 12. Perfis de Compilação

| Perfil | Comando | Uso |
|--------|---------|-----|
| `dev` | `cargo build` | Desenvolvimento, debug rápido |
| `release` | `cargo build --release` | Uso diário, símbolos incluídos |
| `commercial` | `cargo build --profile commercial` | Distribuição: strip + LTO fat + anti-debug |

**`Cargo.toml` (excerpto):**
```toml
[profile.release]
opt-level = 3
codegen-units = 8      # paralelo → build rápido
debug = false

[profile.commercial]
inherits = "release"
strip = true           # remove símbolos
lto = "fat"            # link-time optimization total
codegen-units = 1      # máxima otimização
panic = "abort"        # remove stack unwind info
```

---

## 13. Diagrama de Dependências

```
opendj (app)
├── slint 1.x              — UI declarativa (Slint DSL)
├── opendj-live (live)
│   ├── tiny-skia          — renderer 2D (compositor)
│   ├── ab-glyph           — renderização de texto
│   ├── opus               — codec áudio B2B
│   └── image / jpeg       — decode frames dos celulares
├── ring                   — criptografia Ed25519 (licença)
├── keychain               — Keychain macOS (chave RTMP)
├── serde / serde_json     — serialização de settings
└── opendj-engine (engine)
    ├── rubato             — resampling de áudio
    ├── symphonia          — decode de áudio (MP3, FLAC, AAC…)
    └── htdemucs           — separação de stems (AI)
```

**FFmpeg** não é uma dependência Rust — é um processo separado chamado via
`std::process::Command`. O openDJ o descobre no PATH / Homebrew em runtime.

---

## Comandos úteis de desenvolvimento

```bash
# Build rápido (dev)
cargo build -p opendj

# Check sem build (mais rápido para verificar erros)
cargo check -p opendj

# Rodar testes do crate live
cargo test -p opendj-live

# Rodar apenas testes de output (com ffmpeg real)
cargo test -p opendj-live -- output

# Preview do compositor (gera PNGs em /tmp/preview/)
LIVE_PREVIEW_DIR=/tmp/preview cargo test -p opendj-live -- writes_preview_pngs

# Build de distribuição (comercial)
cargo build --profile commercial -p opendj

# Hardware ID (para gerar licença)
./target/release/opendj --machine-id

# Gerar par de chaves Ed25519
./target/release/opendj --keygen

# Criar licença para uma máquina
./target/release/opendj --create-license MACH-XXXX-XXXX-XXXX-XXXX
```

---

*Arquitetura técnica SMAYKDJ — openDJ edição DJ para lives no TikTok*
