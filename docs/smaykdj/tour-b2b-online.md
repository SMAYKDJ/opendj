# Tour Rápido — B2B Online em 5 Minutos

> Guia passo-a-passo resumido para começar a tocar B2B Online no openDJ.
> Para o manual completo, veja [manual-b2b-online.md](manual-b2b-online.md).

---

## Antes de começar

- [ ] openDJ (edição SMAYKDJ) compilado nos **dois** computadores
- [ ] Ambos na **mesma rede Wi-Fi** (ou porta `7498` liberada no roteador do host para internet)
- [ ] ffmpeg instalado (`brew install ffmpeg`) se for usar a live no TikTok

---

## 🎧 DJ 1 — Anfitrião (Host)

### Passo 1: Abrir o painel LIVE

Clique no botão **LIVE** na barra superior do openDJ.

```
┌───────────────────────────────────────────┐
│  🎵 openDJ              [LIVE] [MAP MIDI] │
└───────────────────────────────────────────┘
                             ▲
                        clique aqui
```

### Passo 2: Criar a sala

Na seção **B2B ONLINE**, clique em **Criar Sala B2B**.

Um código de 4 dígitos aparece na tela:

```
╔══════════════════════════════════╗
║  Código da Sala: 7842  [Copiar]  ║
║  IP público: 201.86.12.34:7498   ║
║  Status: Aguardando convidado…   ║
╚══════════════════════════════════╝
```

### Passo 3: Compartilhar o código

Clique em **Copiar** e envie pelo WhatsApp, Discord, etc:

```
Código: 7842 (IP internet: 201.86.12.34:7498)
```

### Passo 4: Escolher seu deck

No seletor **MEU DECK**, escolha **Deck A** ou **Deck B**.

---

## 🎧 DJ 2 — Convidado (Guest)

### Passo 5: Entrar com o código

1. Clique em **LIVE** na barra superior.
2. Na seção **B2B ONLINE**, clique em **Entrar com Código**.
3. Digite o código **7842**.
4. Clique em **Conectar**.

```
╔════════════════════════════════════╗
║  Código: [7842]  [Conectar]        ║
║  Status: Procurando na rede…  ⏳   ║
╚════════════════════════════════════╝
```

### Passo 6: Conectado!

O openDJ encontra o anfitrião automaticamente via UDP na rede local.

```
╔════════════════════════════════════╗
║  Status: ✅ Conectado               ║
║  Parceiro: DJ SMAYK                ║
╚════════════════════════════════════╝
```

> **Não encontrou?** Depois de 15 s, o campo de IP manual aparece.
> Digite o IP do anfitrião (ex: `192.168.1.42` na LAN ou `201.86.12.34:7498` na internet).

---

## 🎶 Tocando juntos

### Passo 7: Carregar faixas

Cada DJ carrega faixas no **seu deck**.

A **waveform** é enviada automaticamente para o parceiro (~30 KB).

```
╔═══════════════════════════════════════╗
║  Deck A                              ║
║  ┌─ B2B · DJ SMAYK ─┐                ║
║  │▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒░░░│  🎵 Track 1   ║
║  └───────────────────┘                ║
╠═══════════════════════════════════════╣
║  Deck B                              ║
║  ┌─ B2B · DJ CONVIDADO ─┐            ║
║  │▓▓▓▓▒▒▒▒▒▒▒▒░░░░░░░░│  🎵 Track 2  ║
║  └───────────────────────┘            ║
╚═══════════════════════════════════════╝
```

### Passo 8: Ouvir o parceiro

O áudio do parceiro é transmitido via **Opus** automaticamente e misturado no master. Não precisa configurar nada.

### Passo 9: Mixar!

Ambos os decks estão **100% livres**. Você pode mexer nos controles de ambos os decks — o badge "B2B · Nome" indica quem carregou a faixa, mas não trava nada.

> **Dica de batalha:** Fique atento! Se você mexer no deck errado, o público percebe. 😉

---

## 📱 Transmitir no TikTok (opcional)

### Passo 10: Abrir a janela de projetor

No painel LIVE do **anfitrião**, clique em **Abrir janela (TikTok Studio)**.

Uma janela vertical 9:16 se abrirá com tudo composto:

```
┌──────────────────┐
│   📷DJ1 │ 📷DJ2  │  ← câmeras
├──────────────────┤
│ PRÓXIMO: DJ2 42s │  ← aviso
├──────────────────┤
│                  │
│    openDJ UI     │  ← decks
│                  │
├──────────────────┤
│   (chat TikTok)  │
└──────────────────┘
```

### Passo 11: Capturar no TikTok LIVE Studio

1. Abra o **TikTok LIVE Studio**.
2. **Adicionar Fonte › Captura de Janela**.
3. Selecione **"openDJ - TikTok Live"**.
4. Inicie a live! 🎉

---

## ❓ Problemas rápidos

| Problema | Solução |
|----------|---------|
| Não encontra o anfitrião | Digite o IP manualmente depois do timeout |
| Waveform não aparece | Recarregue a faixa (a waveform é enviada ao carregar) |
| Áudio não chega | Verifique se o master do parceiro tem volume > 0 |
| Firewall bloqueando | Ajustes do Sistema › Rede › Firewall › Permitir openDJ |
| Pela internet | Libere a porta `7498` TCP no roteador do host |

---

## Resumo visual do fluxo

```
 DJ 1 (Host)                              DJ 2 (Guest)
 ───────────                              ─────────────
 1. LIVE › Criar Sala ──────────────────►
 2. Copiar código ─── WhatsApp ──────────► 3. Colar código
                                           4. Conectar
       ◄──── Auto-descoberta UDP ──────────
       ◄──── ou IP manual ─────────────────
 5. ✅ Conectado ◄──────────────────────► 5. ✅ Conectado
 6. Carregar faixa ──── waveform ───────► 6. Vê a waveform
 7. Tocar ──── áudio Opus ──────────────► 7. Ouve no master
 8. Câmera ──── JPEG ───────────────────► 8. Vê no compositor
 9. Mixar juntos! 🎉
```

---

> **Manual completo:** [manual-b2b-online.md](manual-b2b-online.md)
>
> **Roadmap técnico:** [b2b-online.md](b2b-online.md)
>
> **Battle Live solo:** [live-tiktok.md](live-tiktok.md)
