# Battle Live: transmissão para o TikTok

Transmitir uma sessão do openDJ direto para a live do TikTok, com câmeras de
celular e um diretor de cenas automático, sem precisar do OBS.

**Estado (2026-11-18):** implementado no branch `smaykdj` (botão **LIVE**),
aguardando o primeiro teste no Mac com celulares e chave do TikTok. Precisa
do ffmpeg (`brew install ffmpeg`).

## Como usar

1. Clique em **LIVE** na barra superior.
2. Escolha **Solo** ou **B2B** e escreva os nomes.
3. Em cada celular, leia o QR code com a câmera. Na primeira vez o navegador
   avisa que a conexão não é particular: toque em *Mostrar detalhes* ›
   *Visitar o site* (iPhone) ou *Avançado* › *Continuar* (Android). Depois
   toque em **Ligar câmera** e permita o acesso.
4. **Transmissão:**
   - **Com TikTok LIVE Studio:** clique no botão **Abrir janela (TikTok Studio)** no painel LIVE (ou toque na prévia da imagem). Uma janela dedicada vertical 9:16 ("openDJ - TikTok Live") se abrirá a 30 FPS. No TikTok LIVE Studio, clique em **Adicionar Fonte › Captura de Janela (Window Capture)** e selecione a janela do openDJ!
   - **Direto pelo openDJ (sem TikTok Live Studio):** cole a URL do servidor e a chave de transmissão (do tiktok.com/live/producer), deixe **Gravar** ligado se quiser o MP4 e clique em **Entrar ao vivo**. Feche o painel antes para ele não cobrir a tela da mixagem.

## Fluxo

1. O DJ clica em **TRANSMITIR LIVE** na barra superior. Fora isso, a tela do
   openDJ continua exatamente como é hoje.
2. Abre uma janela com:
   - a escolha do modo: **Solo · 1 DJ** ou **B2B · 2 DJs**;
   - um QR code para os celulares lerem (mesma rede Wi-Fi do notebook);
   - os dois lugares de celular, com o estado de cada um (conectado ou
     aguardando);
   - o intervalo de alternância das câmeras;
   - o botão **Entrar ao vivo**.
3. Só a transmissão recebe a imagem montada. Na tela do notebook nada muda.

## O que o público vê (vertical 9:16, 1080×1920)

De cima para baixo:

| Faixa | Conteúdo |
| --- | --- |
| Topo (≈7%) | Reservado para o TikTok: conta, botão Seguir, LIVE e espectadores |
| Câmeras | B2B: os dois DJs lado a lado com "VS" no meio, ou um só, conforme a cena. Solo: o DJ ocupando a largura, com o ângulo das mãos numa janela pequena |
| Aviso | B2B: "PRÓXIMO: [DJ] · faixa atual acaba em 0:42" quando a faixa está perto do fim |
| Tela do openDJ | A tela do notebook, inteira, logo abaixo das câmeras |
| Base (≈30%) | Reservado para os comentários, presentes e campo de comentar do TikTok; nada importante fica ali |

## Modos

### Solo · 1 DJ
- Celular 1 filma o rosto; celular 2 (opcional) filma as mãos na controladora.
- Não há "próximo DJ". O diretor alterna entre os ângulos a cada intervalo e
  no drop. Com um celular só, a câmera fica fixa.

### B2B · 2 DJs
- Um celular por DJ.
- Três cenas: só o DJ da vez, os dois (batalha), só o próximo DJ.

## Diretor de cenas automático (B2B)

Não depende do crossfader: muitos DJs mixam pelo volume do canal.

- **Quem é o próximo:** a faixa com **menos tempo restante** é a que vai sair;
  o DJ do outro deck é quem entra.
- **O que conta como "soando":** o nível real de cada canal no master (acima
  de −30 dB por padrão). Vale para quem usa o volume do canal e para quem usa
  o crossfader.
- **Regras:**
  - só a faixa do DJ da vez soando → cena **só o DJ da vez**;
  - faixa da vez a menos de 60 s do fim (configurável), ou as duas soando
    juntas na mixagem → cena **os dois**;
  - a faixa que saiu parou de soar → cena **só o próximo DJ**;
  - nada muda por muito tempo → alterna a cada 16 compassos (configurável,
    também em segundos);
  - tempo mínimo em cada cena: 4 compassos, para não piscar;
  - tecla ou pad MIDI escolhe a cena na mão e pausa o automático.
- **Suposição a confirmar:** a câmera segue o **DJ**, não o deck, porque em
  B2B os DJs nem sempre ficam no mesmo deck. Se cada DJ usar sempre o mesmo
  deck, cada câmera pode seguir o seu deck diretamente.

## TikTok

- O caminho realista é a **chave de transmissão** (servidor RTMP + chave),
  colada no openDJ. Não foi encontrada uma forma pública de um aplicativo de
  terceiros fazer login no TikTok e abrir a live sozinho; as alternativas
  conhecidas são o TikTok LIVE Studio e ferramentas aprovadas pelo TikTok.
- Segundo um guia de terceiros (TikFinity, jan. 2026), a chave pede pelo
  menos 1.000 seguidores, 18 anos ou mais, conta sem punições, e não está
  disponível em todos os países.
- Saída: vídeo vertical 1080×1920, 30 fps, H.264, com o áudio da mixagem
  master em AAC.

## Como foi construído

| Etapa | Onde | Como |
| --- | --- | --- |
| 1. Câmeras dos celulares | `crates/live/src/phones.rs`, `camera.html` | Servidor HTTPS na porta 7497 (certificado feito no notebook e guardado), uma chave aleatória por sessão no QR code, o celular manda JPEG pelo WebSocket (até 24 por segundo, lado maior 960 px) |
| 2. Tela do openDJ | `crates/app/src/live.rs` | `take_snapshot` da janela, 10 vezes por segundo ao vivo e 2,5 na prévia, reduzida para o tamanho em que aparece |
| 3. Imagem vertical | `crates/live/src/compose.rs` | 720 × 1280 desenhado com tiny-skia e a fonte Inter, no layout do design |
| 4. Diretor de cenas | `crates/live/src/director.rs` | Regras acima, lendo `VuMeter`, `volume`, `crossfader`, `duration`, `playposition` e `bpm` |
| 5. Envio | `crates/live/src/output.rs`, `session.rs` | ffmpeg com a imagem pela entrada padrão e o master por um socket local; H.264 pelo VideoToolbox do Mac e AAC; RTMP (FLV), MP4 fragmentado ou os dois |

## Fontes

- [How to Get Your TikTok Stream Key in 2026 (TikFinity)](https://blog.tikfinity.com/?p=7010)
- [Social media safe zones 2026 guide (AdaptlyPost)](https://adaptlypost.com/blog/social-media-safe-zones-2026-complete-guide)
