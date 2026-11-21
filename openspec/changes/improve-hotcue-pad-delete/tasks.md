# Tasks

## 1. Botão × no HotcueCell

- [ ] 1.1 Em `HotcueCell` (`deck.slint`), dentro do bloco `if set:`, adicionar um `Rectangle` posicionado absolutamente no canto superior direito do pad (x: parent.width - width - 2px, y: 2px, ~14×14 px) com um `Text` "×" na cor `tint.with-alpha(0.6)`, visível apenas quando `touch.has-hover` é verdadeiro
- [ ] 1.2 Adicionar um `TouchArea` sobre o × que chama `Controls.set-value(clear-index, 1)` no `down` e `Controls.set-value(clear-index, 0)` no `up`, e interrompe a propagação para o `touch` pai (para não tocar o cue ao mesmo tempo)
- [ ] 1.3 Verificar que Shift+clique no corpo do pad continua funcionando sem interferência do novo TouchArea
