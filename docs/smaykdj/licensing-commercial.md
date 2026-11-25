# Arquitetura Comercial, Anti-Pirataria e Licenciamento Automático

> **openDJ — Edição Comercial SMAYKDJ**  
> Modelo de monetização para o nicho de DJs no TikTok e criadores de conteúdo ao vivo.

---

## 1. Visão Geral dos 4 Pilares

Para comercializar o software mantendo o código seguro e a experiência do usuário fluida, a arquitetura foi dividida em 4 pilares:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            ARQUITETURA COMERCIAL                             │
│                                                                              │
│  [1. Blindagem do Binário]      [2. Hardware Fingerprint]                    │
│  • Strip de símbolos (strip)    • Leitura de IOPlatformSerialNumber / UUID   │
│  • LTO Fat & Codegen-units=1    • Hash SHA-256 (MACH-XXXX-XXXX-XXXX-XXXX)    │
│  • Anti-Debugging (sysctl)      • Chave amigável atrelada à máquina          │
│  • Ofuscação de Strings         • Imutável por reboot                        │
│                                                                              │
│  [3. Criptografia Ed25519]      [4. Gateway Automático (Pendente Ativação)]  │
│  • Assinatura assimétrica       • Webhook Mercado Pago (PIX) / Stripe        │
│  • Validação 100% offline       • Disparo de chave por e-mail em <10s        │
│  • Chave pública no app         • Modo Soft/Testes (Livre para uso atual)   │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Estado Atual: Modo de Testes Livres (Soft Mode)

Conforme planejado, **o software está atualmente com a trava comercial em modo "Soft / Testes Livres"**:

```rust
// crates/app/src/license.rs
pub const ENFORCE_LICENSE: bool = false; // Modo livre ativo para testes
```

* **O que isso significa:**
  * Todas as funções do openDJ (Battle Live vertical, projetor TikTok, áudio Opus, B2B Online) estão **100% liberadas**.
  * No painel LIVE, o badge exibe `"TESTES LIBERADOS"`.
  * Você pode testar e validar o software em transmissões reais pelo tempo que desejar sem nenhum risco de interrupção ou bloqueio.
  * O campo de ativação e validação de chaves já está funcionando para testes: você pode inserir uma chave gerada pelo sistema e testar a validação matemática a qualquer momento.

* **Quando decidir iniciar as vendas:**
  * Basta alterar `ENFORCE_LICENSE = true;` no arquivo `crates/app/src/license.rs`.
  * O software passará a exigir a chave comercial para desbloquear a transmissão para o TikTok LIVE.

---

## 3. Blindagem contra Engenharia Reversa (Etapa 1)

O código Rust é compilado nativamente para binário de máquina macOS (Mach-O ARM64 / Apple Silicon). Para dificultar a descompilação por ferramentas como Ghidra, IDA Pro ou Hopper:

### 3.1. Configuração do `Cargo.toml`
```toml
[profile.release]
opt-level = 3        # Otimização máxima
lto = "fat"          # Link-Time Optimization agressivo através de todos os módulos
codegen-units = 1    # Bloco único para forçar inline profundo de funções
strip = true         # Elimina completamente tabelas de símbolos e caminhos de debug
panic = "abort"      # Remove código de unwinding e mensagens de erro do sistema
```

### 3.2. Módulo de Segurança (`crates/app/src/security.rs`)
* **Detecção de Depuradores (`is_debugger_present`):** Consulta a flag `P_TRACED` da estrutura de processo do kernel macOS (`sysctl`) para identificar se depuradores como LLDB ou Frida estão inspecionando o processo.
* **Bloqueio de Anexação (`deny_debugger_attach`):** Suporte nativo à syscall `PT_DENY_ATTACH` do macOS, que encerra o processo se alguém tentar plugar um depurador.
* **Ofuscação de Strings (`ObfStr`):** Strings críticas (como marcadores de assinatura e endpoints) são mascaradas com chaves XOR em tempo de compilação e descriptografadas apenas na memória volátil.

---

## 4. Hardware Fingerprint — Machine ID (Etapa 2)

Para evitar que uma única licença seja compartilhada com dezenas de computadores, o openDJ gera um **Machine ID** exclusivo para o hardware do cliente:

* **No macOS:** extrai o número de série da placa-mãe (`IOPlatformSerialNumber`) ou UUID de hardware do IOKit (`ioreg -rd1 -c IOPlatformExpertDevice`).
* **Fallback seguro:** combina identificadores do sistema (`hw.model`, processador, variáveis de ambiente).
* **Formato gerado:** `MACH-A1B2-C3D4-E5F6-7890` (16 caracteres hexadecimais em 4 grupos de 4).
* **Vantagens:** Não muda com atualizações do macOS, não depende de conexão à internet e não expõe dados sensíveis do computador do cliente.

---

## 5. Validador Criptográfico Ed25519 (Etapa 3)

O sistema utiliza criptografia assimétrica de curva elíptica **Ed25519** (o mesmo padrão usado em autenticação bancária e chaves SSH modernas):

1. **Chave Privada:** Fica **apenas no seu servidor de vendas**. Ela nunca entra no código do openDJ distribuído aos clientes.
2. **Chave Pública:** Fica embutida no binário do openDJ (`crates/app/src/license.rs`).
3. **Formato da Chave de Licença:**
   ```
   ODJ1.<PAYLOAD_JSON_BASE64>.<ASSINATURA_ED25519_BASE64>
   ```
4. **Validação Offline:** O openDJ do cliente verifica a assinatura matematicamente. Se a assinatura bater com a chave pública e o Machine ID for o mesmo da máquina, a licença é aceita. **O cliente não precisa ter internet para tocar.**

---

## 6. Comandos CLI Administrativos Embutidos

Você pode usar o próprio executável do openDJ no Terminal para gerenciar chaves e gerar licenças de teste:

### Ver o Machine ID do seu computador
```bash
./target/release/opendj --machine-id
# Exibe: MACH-B42C-1F8A-59D0-934A
```

### Gerar um novo par de chaves Mestre (Ed25519)
```bash
./target/release/opendj --keygen
```
*Gera a Chave Pública (para embutir no openDJ) e a Chave Privada (para configurar no backend de vendas).*

### Gerar uma chave de licença para um cliente
```bash
./target/release/opendj --create-license "dj@cliente.com" "MACH-B42C-1F8A-59D0-934A" "TIKTOK_PRO"
```
*Gera instantaneamente a chave no formato `ODJ1.ey...` pronta para colar no painel LIVE ou enviar por e-mail.*

---

## 7. Gateway de Pagamento e Emissão Automática (Etapa 4)

Quando você for iniciar a venda do software, o fluxo automático funciona em 3 passos:

```
[ Checkout Kiwify / Mercado Pago / Hotmart ]
  1. Cliente compra via PIX ou Cartão e digita seu Machine ID.
                    │
                    ▼ (Webhook: payment_approved)
[ Servidor Serverless (Cloudflare Worker gratuito) ]
  2. Recebe o Machine ID e o e-mail do comprador.
  3. Assina a licença com a CHAVE PRIVADA via Ed25519.
  4. Envia o e-mail com a chave de ativação via Resend / SendGrid.
                    │
                    ▼
[ E-mail do DJ em menos de 10 segundos ]
  "Parabéns! Sua chave openDJ TikTok PRO é: ODJ1.ey..."
```

### Código Completo do Webhook (Cloudflare Worker / Node.js)
Salve este código no seu Cloudflare Worker ou API em Node.js quando for ativar as vendas:

```javascript
import crypto from "node:crypto";

export default {
  async fetch(request, env) {
    if (request.method !== "POST") return new Response("OK");

    const payload = await request.json();

    // 1. Validar se o pagamento foi aprovado (Exemplo: Mercado Pago ou Kiwify)
    if (payload.status === "approved" || payload.order_status === "paid") {
      const email = payload.payer.email || payload.customer.email;
      const machineId = payload.custom_fields?.machine_id || payload.metadata?.machine_id;

      if (!machineId) {
        return new Response("Machine ID ausente", { status: 400 });
      }

      // 2. Montar os dados da licença
      const licenseData = {
        email: email,
        machine_id: machineId,
        tier: "TIKTOK_PRO",
        issued_at: Math.floor(Date.now() / 1000)
      };

      const licenseJson = Buffer.from(JSON.stringify(licenseData));

      // 3. Assinar com a Chave Privada Ed25519 guardada nos Secrets do Worker
      const privateKey = crypto.createPrivateKey({
        key: Buffer.from(env.ED25519_PRIVATE_KEY_B64, "base64"),
        format: "der",
        type: "pkcs8"
      });

      const signature = crypto.sign(null, licenseJson, privateKey);

      // 4. Formatar a chave final
      const b64Data = licenseJson.toString("base64url");
      const b64Sig = signature.toString("base64url");
      const licenseKey = `ODJ1.${b64Data}.${b64Sig}`;

      // 5. Enviar e-mail automático via Resend
      await fetch("https://api.resend.com/emails", {
        method: "POST",
        headers: {
          "Authorization": `Bearer ${env.RESEND_API_KEY}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          from: "openDJ <licencas@seudominio.com>",
          to: [email],
          subject: "Sua Chave de Ativação openDJ TikTok PRO!",
          html: `
            <h2>Obrigado pela sua compra!</h2>
            <p>Seu openDJ TikTok PRO já está liberado para a máquina: <b>${machineId}</b></p>
            <p>Sua chave de ativação:</p>
            <pre style="background:#f4f4f4;padding:12px;border-radius:6px;word-break:break-all;">${licenseKey}</pre>
            <p>Abra o openDJ, clique em <b>LIVE</b>, cole a chave no campo <b>Validar Chave</b> e pronto!</p>
          `
        })
      });

      return new Response(JSON.stringify({ success: true, key: licenseKey }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    return new Response("Ignorado", { status: 200 });
  }
};
```

---

## 8. Resumo da Entrega

| Item | Status | Onde está implementado |
|---|---|---|
| **1. Blindagem de Binário** | Implementado | `Cargo.toml` (`[profile.release]`), `crates/app/src/security.rs` |
| **2. Hardware ID (macOS)** | Implementado | `crates/app/src/license.rs` (`get_machine_id`), CLI `--machine-id` |
| **3. Criptografia Ed25519** | Implementado | `crates/app/src/license.rs`, `crates/app/ui/live.slint`, CLI `--create-license` |
| **4. Gateway / Ativação Obrigatória** | Modo Soft (Ativação pendente para testes) | `ENFORCE_LICENSE = false`, documentação e worker prontos em `licensing-commercial.md` |
