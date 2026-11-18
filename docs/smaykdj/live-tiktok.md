# Battle Live: transmissão para o TikTok

Especificação de um recurso ainda **não implementado**: transmitir uma sessão
do openDJ direto para a live do TikTok, com câmeras de celular e um diretor
de cenas automático, sem precisar do OBS.

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

## O que precisa ser construído

Hoje o openDJ transmite só áudio (rádio pela internet, crate `broadcast`).
Para esta função faltam:

1. Receber o vídeo dos celulares pelo QR code (o openDJ já pareia o celular
   por QR para o controle remoto, crate `remote`).
2. Capturar a tela do openDJ.
3. Montar a imagem vertical com as câmeras e a tela.
4. O diretor de cenas, lendo o nível dos canais e o tempo restante das faixas.
5. Codificar vídeo + áudio master e enviar ao servidor RTMP do TikTok.

Primeira etapa sugerida: os passos 1 a 4, salvando a imagem vertical como
gravação. A transmissão ao TikTok (passo 5) vem depois.

## Fontes

- [How to Get Your TikTok Stream Key in 2026 (TikFinity)](https://blog.tikfinity.com/?p=7010)
- [Social media safe zones 2026 guide (AdaptlyPost)](https://adaptlypost.com/blog/social-media-safe-zones-2026-complete-guide)
