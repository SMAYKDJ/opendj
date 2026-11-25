# Modo Solo — openDJ edição SMAYKDJ

> Um DJ, duas câmeras, uma live vertical perfeita para o TikTok.

---

## Índice

1. [O que é o Modo Solo](#1-o-que-é-o-modo-solo)
2. [Layout da tela (9:16)](#2-layout-da-tela-916)
3. [Configurar as câmeras](#3-configurar-as-câmeras)
4. [Cenas disponíveis](#4-cenas-disponíveis)
5. [Diretor automático de cenas](#5-diretor-automático-de-cenas)
6. [Escolher a cena manualmente](#6-escolher-a-cena-manualmente)
7. [Transmitir ao vivo](#7-transmitir-ao-vivo)

---

## 1. O que é o Modo Solo

O Modo Solo é para quando **você toca sozinho** e quer transmitir ao vivo
no TikTok com visual profissional. Você usa dois celulares como câmeras:

- **Celular 1 (Rosto):** mostra o rosto e a parte superior do corpo
- **Celular 2 (Mãos):** mostra as mãos nas controladoras

Como o celular fica em um suporte e o DJ enquadra como quiser, as duas câmeras
têm **o mesmo tamanho na tela** — sem câmera "principal" e "secundária".

---

## 2. Layout da tela (9:16)

O que o público vê na live (720 × 1280 pixels):

```
┌────────────────────────────┐  ← faixa de topo (TikTok cobre)
│                            │
├─────────────┬──────────────┤  ← linha das câmeras
│             │ ⃝ VS ⃝       │
│  📷 Cel 1   │              │  câmeras iguais, lado a lado
│   Rosto     │  📷 Cel 2   │  com círculo "VS" no centro
│  DJ SMAYK  │   Mãos       │
│             │              │
├─────────────┴──────────────┤
│                            │
│    ████ openDJ ████████    │  ← tela do openDJ (decks, waveform)
│                            │
├────────────────────────────┤
│    barra de votação A|B    │  ← placar de votos (quando ativo)
├────────────────────────────┤  ← faixa de base (TikTok cobre)
│      openDJ  SOLO DJ       │  ← rodapé com modo
└────────────────────────────┘
```

**Faixas cobertas pelo TikTok:** os primeiros ~65 px do topo (nome do usuário,
badges) e os últimos ~65 px do rodapé (comentários e botões). O openDJ deixa
essas áreas livres propositalmente.

---

## 3. Configurar as câmeras

### No openDJ (computador)

1. Clique em **LIVE** na barra superior
2. No campo **Solo · 1 DJ** (switch B2B desligado)
3. Clique em **"Iniciar câmeras"** — aparece um QR code para cada celular

### No celular

1. Abra a câmera do celular e aponte para o QR code
2. Toque no link que aparece — abre uma página no navegador
3. Toque em **"Enviar câmera"** e aceite a permissão da câmera
4. Posicione o celular no suporte na posição desejada

> **Dica:** O celular conecta pelo Wi-Fi da sua casa. Ambos precisam estar
> na mesma rede que o seu Mac.

### Sugestão de posicionamento

```
         [Cel 1 — Rosto]          [Cel 2 — Mãos]
         frontal ou 3/4           de cima, 45°
              📱                       📱
              ↓                        ↓
          ┌───────┐               ┌──────────┐
          │ 😎 DJ │               │ 🎚️ ✋ 🎚️ │
          └───────┘               └──────────┘
```

---

## 4. Cenas disponíveis

No painel LIVE, campo **CENA**:

| Cena | O que mostra |
|------|-------------|
| **Automático** | O diretor troca conforme o áudio |
| **Rosto** | Só o Celular 1 (rosto), ocupando toda a área |
| **Mãos** | Só o Celular 2 (mãos), ocupando toda a área |
| *(ambos)* | Os dois lado a lado — modo padrão no automático |

> Anteriormente havia um layout com câmera grande e PiP pequeno. Agora as
> duas câmeras sempre têm o **mesmo tamanho** quando exibidas juntas.

---

## 5. Diretor automático de cenas

Com **Automático** selecionado, o openDJ troca de cena a cada intervalo
configurado (padrão: 16 compassos). A troca considera o nível de áudio de
cada canal.

**Comportamento:**
- Se houver só uma câmera conectada: exibe só ela em tela cheia
- Se houver dois celulares: alterna entre "ambos juntos" e "só um"
- Nunca troca antes de 4 compassos (evita flicker)

---

## 6. Escolher a cena manualmente

No campo **CENA** do painel LIVE, selecione a cena desejada a qualquer
momento, mesmo durante a live. O diretor automático é suspenso enquanto
uma cena específica está selecionada.

Para voltar ao automático: selecione **Automático**.

---

## 7. Transmitir ao vivo

Com as câmeras configuradas:

1. Ligue o switch **"Transmitir para o TikTok"**
2. Confirme a URL do servidor: `rtmp://push.tiktok.com/live/`
3. Cole a **Chave de transmissão** do TikTok
4. Opcional: ligue **"Gravar"** para salvar uma cópia local em MP4
5. Clique em **"Entrar ao vivo"**

Para mais detalhes sobre RTMP e chaves, veja
[guia-transmissao-rtmp.md](guia-transmissao-rtmp.md).

---

*Documentação SMAYKDJ — openDJ modo Solo para lives no TikTok*
