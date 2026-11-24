# B2B Online Remoto: Transmissão Compartilhada para o TikTok

O modo **B2B Online** permite que dois DJs em computadores e locais diferentes dividam a mesma sessão do openDJ através de uma conexão de rede direta (Peer-to-Peer), compartilhem a visualização das waveforms em tempo real e transmitam uma live conjunta para o TikTok.

---

## Como Funciona o B2B Online

1. **Código de Sala (Sem Contas):**
   - O DJ Anfitrião clica em **Criar Sala B2B** no painel LIVE.
   - O openDJ gera um código de 4 a 6 dígitos (ex: `7842`).
   - O DJ Convidado abre seu openDJ, digita o código `7842` e clica em **Conectar**.
   - Os dois computadores estabelecem uma conexão direta criptografada.

2. **Divisão dos Decks (Deck Lock):**
   - O DJ Anfitrião assume um deck (ex: **Deck A**).
   - O DJ Convidado assume o outro deck (ex: **Deck B**).
   - O controle é exclusivo: nenhum DJ interfere acidentalmente no deck do outro.

3. **Waveform ao Vivo Sincronizada:**
   - Quando o DJ remoto carrega uma faixa, os dados da waveform (~30 KB) são enviados de forma ultraleve.
   - Ambos os DJs veem as formas de onda correndo lado a lado na tela com os grids de compasso e BPM alinhados.

4. **Áudio e Vídeo de Baixa Latência:**
   - O áudio do DJ convidado é transmitido via codec Opus (latência de 30-50 ms).
   - A câmera de cada DJ é posicionada no topo da cena vertical pelo compositor do openDJ.

5. **Transmissão para o TikTok:**
   - O anfitrião abre a janela do projetor (**`openDJ - TikTok Live`**) e a captura no **TikTok LIVE Studio** via *Captura de Janela*.
   - O público assiste aos dois DJs lado a lado no topo e à mixagem dos dois decks acontecendo embaixo.

---

## Arquitetura Técnica

```
[ openDJ - DJ 1 (Host) ]                                 [ openDJ - DJ 2 (Guest) ]
       │                                                              │
       ├─ Deck A (Controle Local)          Deck A (Visualiza DJ 1) ───┤
       ├─ Deck B (Visualiza DJ 2) ◄──────── Deck B (Controle Local) ──┤
       │                                                              │
       ├─── 1. Áudio Opus (30-50ms) ──────────────────────────────────┤
       ├─── 2. Câmera H.264 (720p) ───────────────────────────────────┤
       └─── 3. DataChannel: Waveforms + Posição + Faders ─────────────┘
       │
       ▼
 [ Compositor 9:16 ]
       │
       ├─► Janela de Projetor (TikTok LIVE Studio)
       └─► RTMP Direto (tiktok.com/live/producer)
```

---

## Roadmap de Desenvolvimento

1. **Fase 1 — Interface e Código de Sala:**
   - Interface no painel LIVE com as abas *B2B Local* e *B2B Online*.
   - Campo para gerar e digitar Código de Sala ou IP público direto.
   - Descoberta automática de IP público e travessia NAT via cliente STUN RFC 5389 (Google/Cloudflare).

2. **Fase 2 — Conexão P2P, Câmera e Sincronização de Controles:**
   - [x] Motor TCP de baixa latência (`live_b2b::B2bNetworkSession`) com suporte a Host e Convidado.
   - [x] Auto-descoberta local sem digitação de IP via UDP broadcast (porta `7499`) com fallback manual.
   - [x] Pacote de estado de deck em 30 Hz (`DeckStateMessage`): Play, Cue, Posição da agulha, Faders, EQs e BPM.
   - [x] Espelhamento mútuo instantâneo de controles no `ControlRegistry`.
   - [x] Suporte à câmera do DJ remoto alimentando diretamente o slot 1 do compositor vertical 9:16.
   - [x] Stream de áudio mútuo em tempo real via codec Opus (20 ms @ 128 kbps) integrado ao master mix da live e gravação.

3. **Fase 3 — Modo Colaborativo e Waveforms Compartilhadas:**
   - [x] Modo colaborativo total (EQs, faders e jog wheel livres para ambos os DJs realizarem viradas).
   - [x] Badge visual "B2B · <DJ>" sobre cada deck identificando o responsável pela faixa.
   - [x] Pacote de dados de waveform e metadados transmitido via rede (`TrackLoaded` / `TrackUnloaded`).
   - [x] Sincronização visual em 30 Hz das duas formas de onda correndo lado a lado no openDJ e projetor.

4. **Fase 4 — Validação em Live com TikTok LIVE Studio:**
   - Ajustes de estabilidade para conexões Wi-Fi residenciais.
   - Teste prático de transmissão real entre dois DJs.
