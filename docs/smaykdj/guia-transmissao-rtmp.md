# Guia de Transmissão RTMP — openDJ edição SMAYKDJ

> Transmita ao vivo direto do openDJ para o TikTok, YouTube, Instagram ou
> qualquer servidor RTMP — sem precisar do TikTok LIVE Studio.

---

## Índice

1. [Como funciona](#1-como-funciona)
2. [Pré-requisito: instalar o ffmpeg](#2-pré-requisito-instalar-o-ffmpeg)
3. [Transmitir para o TikTok](#3-transmitir-para-o-tiktok)
4. [Transmitir para o YouTube](#4-transmitir-para-o-youtube)
5. [Transmitir para o Instagram (via OBS ou Restream)](#5-transmitir-para-o-instagram)
6. [Gravar localmente (sem transmitir)](#6-gravar-localmente)
7. [Transmitir e Gravar ao mesmo tempo](#7-transmitir-e-gravar-ao-mesmo-tempo)
8. [Configurações de qualidade](#8-configurações-de-qualidade)
9. [Solução de problemas](#9-solução-de-problemas)

---

## 1. Como funciona

O openDJ inclui um sistema completo de codificação e transmissão integrado:

```
┌─────────────────────────────────────────────────────────────┐
│                        openDJ                               │
│                                                             │
│  📷 Celular 1 ──┐                                           │
│  📷 Celular 2 ──┼──► Compositor 720×1280 ──► ffmpeg ──────► RTMP
│  🖥  Tela      ──┘      (tiny-skia)       H.264+AAC   ou ──► .mp4
│  🎵 Áudio Master ───────────────────────────────────────────┘
│
│  Codec de vídeo: H.264 VideoToolbox (GPU do Mac) → qualidade máxima
│  Codec de áudio: AAC 160 kbps estéreo 48 kHz
│  Resolução: 720 × 1280 (vertical 9:16, padrão TikTok)
│  FPS: 30 quadros por segundo
└─────────────────────────────────────────────────────────────┘
```

**A chave de transmissão é guardada no Keychain do macOS**, não em arquivos de
configuração — fica protegida pelo seu login.

---

## 2. Pré-requisito: instalar o ffmpeg

O ffmpeg **não** vem junto com o openDJ. Precisa instalar uma vez:

```bash
brew install ffmpeg
```

> **Não tem o Homebrew?**
> ```bash
> /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
> ```

**Verificar se está instalado:**
```bash
ffmpeg -version
```

Se aparecer `ffmpeg version 7.x ...`, está pronto.

O openDJ procura o ffmpeg automaticamente em:
- `OPENDJ_FFMPEG` (variável de ambiente, se quiser apontar para um local específico)
- Pasta do executável do openDJ
- `PATH` do sistema
- `/opt/homebrew/bin` (Apple Silicon)
- `/usr/local/bin` (Intel Mac)

---

## 3. Transmitir para o TikTok

### 3.1 Pegar a chave de transmissão

**Opção A — Pelo app TikTok no celular:**
1. Toque no **+** (criar) → **Live**
2. Escolha **Stream de PC** (pode precisar de mais de 1.000 seguidores)
3. Copie a **URL do servidor** e a **Chave de stream**

**Opção B — Pelo TikTok LIVE Studio (desktop):**
1. Abra o TikTok LIVE Studio
2. Vá em **Configurações** → **Stream**
3. Copie a **URL do servidor** e a **Chave**

**Opção C — Pelo navegador:**
1. Acesse [live.tiktok.com/studio](https://live.tiktok.com/studio)
2. Faça login → **Ir ao vivo com software externo**
3. Copie a URL e a chave

> ⚠️ **A chave do TikTok expira após cada live.** Sempre gere uma nova antes
> de transmitir.

### 3.2 Configurar no openDJ

1. Clique em **LIVE** na barra superior do openDJ
2. Em **DESTINO**, certifique-se de que **TikTok** está selecionado
3. A **URL do servidor** já vem pré-preenchida automaticamente:
   ```
   rtmp://push.tiktok.com/live/
   ```
4. **Preenchimento automático da chave:** ao copiar a chave no TikTok (ou pelo navegador), o openDJ detecta automaticamente na área de transferência e preenche o campo sozinho!
   - Uma mensagem verde `✅ Chave do TikTok detectada automaticamente!` aparecerá.
   - Se preferir, você também pode clicar no botão **📋 Colar** ou digitar manualmente.
5. Clique em **Entrar ao vivo**

### 3.3 Encerrar

Clique em **Encerrar · M:SS** para parar. O timer mostra quanto tempo você
ficou ao vivo.

---

## 4. Transmitir para o YouTube

### 4.1 Pegar a chave do YouTube

1. Acesse [studio.youtube.com](https://studio.youtube.com)
2. Clique em **Ir ao vivo** (canto superior direito)
3. Escolha **Transmissão por software**
4. Copie o **URL do servidor** e a **Chave de transmissão**

### 4.2 Configurar no openDJ

No painel LIVE, campo **URL do servidor**:
```
rtmp://a.rtmp.youtube.com/live2/
```
Campo **Chave de transmissão**: cole a chave gerada no YouTube Studio.

> **Observação:** O YouTube exige resolução 1920×1080 (horizontal) para
> transmissões padrão. Para o formato vertical 9:16 do openDJ, use o YouTube
> Shorts Live — verifique se sua conta tem acesso.

---

## 5. Transmitir para o Instagram (Sem limite de seguidores!)

O Instagram oferece suporte oficial a transmissões via software externo
através do **Instagram Live Producer**, disponível para qualquer conta
Profissional (Criador de Conteúdo ou Empresa) **sem exigência de 1.000 seguidores**.

### 5.1 Como pegar a chave do Instagram

1. No computador, acesse no navegador:
   ```
   https://www.instagram.com/live/producer/
   ```
   *(ou entre no instagram.com pelo Mac → menu lateral Criar (+) → Transmissão ao vivo)*
2. Dê um título para a sua live e selecione a audiência (**Público** ou **Prática** para testar primeiro).
3. O Instagram exibirá:
   - **URL do stream:** `rtmps://live-upload.instagram.com:443/rtmp/`
   - **Chave do stream:** sua chave única
4. Copie a chave gerada.

> 💡 **Dica de ouro:** No seletor de audiência do Instagram, escolha **"Prática"**
> nos primeiros testes. Só você verá a transmissão, perfeito para testar o som
> e as câmeras do openDJ antes de entrar ao vivo para seus seguidores!

### 5.2 Configurar no openDJ

1. Clique em **LIVE** na barra superior do openDJ
2. Ligue o switch **"Transmitir ao vivo"**
3. Em **DESTINO**, clique em **Instagram**
4. A **URL do servidor** já é pré-preenchida automaticamente:
   ```
   rtmps://live-upload.instagram.com:443/rtmp/
   ```
5. **Detecção e preenchimento 100% automático da chave:**
   - Na página do **Instagram Live Producer**, basta clicar no botão **"Copiar"** ao lado da Chave do stream.
   - O openDJ monitora a área de transferência em tempo real e **preenche a chave instantaneamente em automático**, exibindo a confirmação `✅ Chave do Instagram detectada automaticamente!`.
   - Você não precisa colar manualmente nem alternar janelas para dar Cmd+V — basta copiar no navegador!
   - *(Opcional)* Se preferir colar manualmente, há também o botão **📋 Colar** direto ao lado do campo.
6. Clique em **Entrar ao vivo** no openDJ
7. Volte ao navegador no **Instagram Live Producer** — você verá a imagem do
   openDJ na prévia. Clique em **Transmitir ao vivo** na página do Instagram.

---

## 6. Gravar localmente

Desligue o switch **"Transmitir ao vivo"** e ligue
**"Gravar em [pasta]"**.

O arquivo é gravado como **MP4 fragmentado** — se o app fechar de repente,
o arquivo já gravado não se perde.

**Pasta padrão:**
```
~/Music/openDJ/Recordings/Live_YYYY-MM-DD_HH-MM-SS.mp4
```

---

## 7. Transmitir e Gravar ao mesmo tempo

Ligue os **dois switches** (Transmitir + Gravar). O ffmpeg usa o muxer `tee`
internamente para escrever o mesmo stream codificado no RTMP e no arquivo
simultaneamente — sem custo duplo de CPU.

```
ffmpeg → tee → [f=flv]rtmp://push.tiktok.com/live/CHAVE
                [f=mp4]~/Music/openDJ/Live_xxxx.mp4
```

---

## 8. Configurações de qualidade

As configurações estão em `crates/live/src/output.rs`. Padrões atuais:

| Parâmetro | Valor padrão | Recomendação TikTok |
|-----------|-------------|---------------------|
| Resolução | 720 × 1280 | ✅ Exato |
| FPS | 30 | ✅ Exato |
| Vídeo (kbps) | 3.000 | ✅ Mínimo 2.500 |
| Áudio (kbps) | 160 | ✅ Exato |
| Codec vídeo | H.264 (VideoToolbox) | ✅ Requerido |
| Codec áudio | AAC | ✅ Requerido |
| Perfil H.264 | High | ✅ Recomendado |
| Formato de saída | FLV (RTMP) | ✅ Padrão |
| GOP (keyframes) | 60 (2 s) | ✅ Padrão |

---

## 9. Solução de problemas

### "⚠ ffmpeg não encontrado"

```bash
brew install ffmpeg
```

Depois reinicie o openDJ.

### Erro "rtmp://..." ao entrar ao vivo

- Verifique se a URL começa com `rtmp://` ou `rtmps://`
- Verifique se a chave de transmissão não está vazia
- A chave do TikTok expira — gere uma nova

### Vídeo travando / queda de FPS

- Feche outros aplicativos pesados
- Reduza o bitrate para 2.000 kbps (edite `video_kbps` em `output.rs`)
- O codec VideoToolbox usa a GPU — verifique se outro app está usando a GPU intensamente

### Sem som na transmissão

- Certifique-se de que há música tocando antes de clicar "Entrar ao vivo"
- O áudio é o master feed do openDJ (mesma saída que vai para os monitores)

### "Acesso não disponível" no TikTok LIVE Studio

Sua conta ainda não tem acesso ao LIVE Studio (exige ≥ 1.000 seguidores ou
aprovação). **Use o RTMP direto pelo openDJ** — não precisa do LIVE Studio.

---

## Referência rápida: servidores RTMP

| Plataforma | URL do servidor |
|------------|----------------|
| TikTok | `rtmp://push.tiktok.com/live/` |
| YouTube | `rtmp://a.rtmp.youtube.com/live2/` |
| Twitch | `rtmp://live.twitch.tv/live/` |
| Facebook | `rtmps://live-api-s.facebook.com:443/rtmp/` |
| Restream | `rtmp://live.restream.io/live/` |

---

*Documentação SMAYKDJ — openDJ edição DJ para lives no TikTok*
