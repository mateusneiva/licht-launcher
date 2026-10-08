# ROADMAP: Licht Launcher

Launcher não oficial de Minecraft: Java Edition para **Windows e Linux**, feito com Tauri v2 (Rust + TypeScript).

**Stack do frontend:** React + TypeScript + Vite, Tailwind CSS, shadcn/ui (Radix UI), Lucide (ícones), TanStack Query (chamadas ao Rust), Zustand (estado global), TanStack Virtual (listas grandes), Vitest + Testing Library (testes).

> **Como usar:** peça sempre **um item por vez** ("faça o item 3.2"). Fluxo de cada item: **plano → aprovação → implementação → PR → revisão**. Siga o `CLAUDE.md`.

Legenda: `[ ]` pendente · `[x]` feito · 🛑 = decisão que exige aprovação explícita antes de codar · 📚 = conceitos de Rust que aparecem

---

## Identidade e convenções

| Item | Valor |
|---|---|
| Nome exibido | **Licht Launcher** (*Licht* = "luz" em alemão; pronúncia adotada: "líkt") |
| Repositório / pacotes | `licht-launcher` |
| Binário / comando | `licht` |
| Crate do core | `licht-core` |
| Símbolo (logo) | um lírio feito de luz |
| Aviso obrigatório | "Projeto não oficial, não afiliado à Mojang Studios nem à Microsoft" |

> 🛑 Antes do item 0.1: confirmar domínio (ex.: `licht.app`, `getlicht.com`, ou um `.com.br` no registro.br) e o *bundle identifier* do app (ex.: `app.licht.launcher`).

### Codinomes de versão

Cada release `0.x` recebe o nome de uma flor, em alemão, nesta ordem:

| Versão | Codinome | Significado |
|---|---|---|
| 0.1 | **Lilie** | lírio |
| 0.2 | **Iris** | íris |
| 0.3 | **Rose** | rosa |
| 0.4 | **Immergrün** | vinca |
| 0.5 | **Anemone** | anêmona |
| 0.6 | **Narzisse** | narciso |
| 0.7 | **Edelweiß** | edelvais |

O codinome aparece na tela "Sobre", no título da release no GitHub e no nome do instalador (ex.: `Licht-Launcher-0.1.0-Lilie`). A partir da 0.8, a lista continua (🛑 definir juntos).

---

## Fase 0. Fundação

**Objetivo:** repositório pronto, com qualidade automatizada desde o primeiro commit.

- [ ] **0.1** Criar o workspace: `crates/licht-core` (lib), `src-tauri` (app), `src/` (frontend TS + Vite).
  - 🛑 Gerenciador de pacotes (pnpm) e estrutura de pastas do frontend (`src/components/ui`, `src/features/*`, `src/lib`).
  - 📚 Cargo workspace, crates lib vs bin.
- [x] **0.2** Tooling: `rustfmt`, `clippy -D warnings`, Biome (lint e formatação), TypeScript estrito, Vitest + Testing Library.
- [ ] **0.3** CI (GitHub Actions) com matriz **windows-latest + ubuntu-latest**: fmt, clippy, testes Rust e frontend, build do Tauri.
- [ ] **0.4** Estrutura de logs (`tracing`) e tipo de erro base (`thiserror`) no core.
  - 📚 `Result`, o operador `?`, `thiserror` vs `anyhow`.
- [ ] **0.5** `ARCHITECTURE.md` curto descrevendo as camadas (core / tauri / frontend).
- [ ] **0.6** Licença do projeto e aviso de "não oficial" no README.
  - 🛑 Licença (MIT, Apache-2.0, GPL...).
- [ ] **0.7** **Design system base**: Tailwind + shadcn/ui configurados, **tokens de design** (cores, raios, espaçamento, tipografia) como variáveis CSS num único arquivo, tema claro/escuro, ícones Lucide e componentes-base (Button, Input, Dialog, Tabs, Progress, Toast, Select, Tooltip, ScrollArea). Página `/styleguide` (somente em dev) mostrando todos eles.
  - 🛑 **Versão do Tailwind (v3 ou v4):** testar em WebKitGTK mais antigo (ex.: Ubuntu 22.04) antes de decidir. Se houver problema, usar v3.
  - 🛑 Direção visual: paleta, tipografia, claro/escuro, nível de minimalismo (ponto de partida: lírio de luz, tons suaves).
  - Evitar efeitos pesados (`backdrop-filter`, blurs e sombras grandes) por causa do WebKitGTK no Linux.

**Pronto quando:** o app Tauri abre uma janela vazia em Windows e Linux, o CI passa verde e a página `/styleguide` renderiza igual nos dois sistemas.

---

## Fase 1. Dados da Mojang (parsing)

**Objetivo:** ler e entender os manifestos, sem rede nos testes.

- [ ] **1.1** Tipos `serde` para `version_manifest_v2` + cliente HTTP (`reqwest`) para buscá-lo.
  - Fixture: salvar um manifesto real em `tests/fixtures`.
  - 📚 `serde`, `struct`/`enum`, `async/await` com `tokio`.
- [ ] **1.2** Tipos para o **JSON de uma versão** (libraries, arguments, assetIndex, mainClass, downloads, javaVersion). Testar com fixtures de versões antigas (ex.: 1.8, 1.12) e novas (1.20+, 1.21).
  - 🛑 Estratégia para versões com formato antigo (`minecraftArguments`) vs novo (`arguments`).
- [ ] **1.3** Avaliador de **`rules`** (os/arch/features) com testes cobrindo Windows, Linux e arquiteturas diferentes.
  - 📚 `match`, `Option`, traits simples.
- [ ] **1.4** Tipos e leitura do **asset index**.

**Pronto quando:** um teste carrega o JSON de qualquer versão fixture e lista as libraries aplicáveis ao SO atual.

---

## Fase 2. Downloads

**Objetivo:** baixar tudo de forma rápida, verificada e retomável.

- [ ] **2.1** Downloader de um arquivo: stream para disco, **verificação SHA1**, escrita atômica (`.part` → rename), retry com backoff.
  - 📚 Ownership de buffers, `Path`/`PathBuf`, `AsyncRead`.
- [ ] **2.2** Fila paralela com limite de concorrência (ex.: 8-16) e agregação de progresso.
  - 🛑 Limite padrão de concorrência e política de retry.
  - 📚 `tokio::spawn`, `Semaphore`, canais (`mpsc`).
- [ ] **2.3** Estrutura de diretórios e **cache compartilhado** (libraries, assets por hash) fora das instâncias.
  - 🛑 Layout em disco (compatível ou não com o `.minecraft` oficial).
  - Usar `directories` para caminhos por SO.
- [ ] **2.4** Instalador de versão: dado um ID de versão, baixa client.jar + libraries + assets e pula o que já está válido.

**Pronto quando:** instalar a mesma versão duas vezes baixa tudo na primeira e nada na segunda.

---

## Fase 3. Java

**Objetivo:** ter o Java certo para cada versão do jogo, sem o usuário instalar nada.

- [ ] **3.1** Descobrir o runtime exigido (`javaVersion` do JSON da versão).
- [ ] **3.2** Baixar e instalar o runtime da Mojang (manifesto `java-runtime`) ou Adoptium/Temurin.
  - 🛑 Qual fonte usar como padrão e se o usuário pode apontar um Java próprio.
- [ ] **3.3** Detectar Javas já instalados (opcional) e validar a versão.

**Pronto quando:** pedir "1.21" e "1.8" resulta em dois runtimes distintos, corretos e utilizáveis.

---

## Fase 4. Montagem e execução do jogo

**Objetivo:** abrir o Minecraft.

- [ ] **4.1** Extração de **natives** por plataforma para uma pasta temporária por execução.
  - 📚 `zip`, tratamento de erros de I/O.
- [ ] **4.2** Montagem do **classpath** (separador `;` no Windows, `:` no Linux) e do comando completo: argumentos JVM + argumentos do jogo, com substituição de variáveis (`${auth_player_name}`, `${game_directory}`, `${assets_root}`...).
- [ ] **4.3** Execução do processo (`std::process`/`tokio::process`), captura de stdout/stderr, detecção de fim/crash.
- [ ] **4.4** CLI de teste (`licht`) em `licht-core` (exemplo/binário) com **auth offline apenas para desenvolvimento**.
  - 🛑 Se o modo offline fica restrito a builds de dev ou vira recurso (implica decisões legais/de produto).

### 🏁 Marco 1: o jogo abre

Instalar e abrir **vanilla 1.21.x** e **uma versão antiga (1.8.9)** em Windows e Linux, a partir do CLI.

---

## Fase 5. Interface básica

**Objetivo:** usar tudo isso por uma UI.

- [ ] **5.1** Commands do Tauri (`list_versions`, `install_version`, `launch`), finos, apenas chamando o core. Tipos compartilhados com o frontend via `ts-rs`/`specta`.
  - 🛑 Contrato (nomes e tipos) dos commands e eventos.
  - 📚 `#[tauri::command]`, `State`, `Arc`/`Mutex`.
- [ ] **5.2** Eventos de progresso (download, instalação, status do jogo) emitidos do Rust para o frontend, com **throttling** (~100 ms) para não saturar a UI.
- [ ] **5.3** Tela de versões: listar (lista **virtualizada**), filtrar (release/snapshot), instalar, jogar, barra de progresso. Usar apenas componentes do design system (item 0.7).
- [ ] **5.4** Console de logs do jogo em tempo real.
- [ ] **5.5** Tratamento de erros amigável na UI (rede fora, disco cheio, SHA1 inválido).
- [ ] **5.6** Identidade visual final: logo (lírio de luz), ícone do app em todos os tamanhos e tela "Sobre" com versão e codinome. A paleta e os tokens já existem desde o item 0.7.
  - 🛑 Direção do logo e do ícone.

**Pronto quando:** dá para instalar e jogar uma versão apenas pela interface.

---

## Fase 6. Instâncias

**Objetivo:** perfis isolados.

- [ ] **6.1** Modelo de instância (nome, versão, pasta própria, RAM mín/máx, argumentos JVM extras, resolução), persistido em JSON versionado.
  - 🛑 Formato do arquivo de configuração e estratégia de migração.
- [ ] **6.2** CRUD de instâncias (criar, duplicar, renomear, excluir) no core + UI.
- [ ] **6.3** Configurações globais (pasta de dados, Java, concorrência de download, tema).
- [ ] **6.4** Abrir pasta da instância no gerenciador de arquivos do SO.

---

## Fase 7. Login Microsoft

**Objetivo:** contas legítimas.

- [ ] **7.1** Registrar app no Azure e solicitar aprovação de acesso à API de serviços do Minecraft. **Vale iniciar este pedido cedo**, em paralelo às fases anteriores, pois pode demorar.
  - 🛑 Tudo desta fase: fluxo (device code recomendado), escopos, onde guardar tokens.
- [ ] **7.2** Fluxo OAuth device code → Xbox Live → XSTS → Minecraft Services → perfil (nome, UUID, skin).
- [ ] **7.3** Armazenamento seguro de tokens com o **keyring do SO** (Credential Manager / Secret Service). Nunca em texto puro, nunca em logs.
- [ ] **7.4** Renovação automática (refresh token), múltiplas contas, logout.
- [ ] **7.5** Verificar se a conta possui o jogo e mensagens de erro claras.

### 🏁 Marco 2: launcher utilizável com conta real

---

## Fase 8. Mod loaders

- [ ] **8.1** **Fabric** (API de metadados): instalar loader numa instância.
- [ ] **8.2** **Quilt** (mesma abordagem).
- [ ] **8.3** **NeoForge/Forge** (rodar instalador/processors).
  - 🛑 Se Forge/NeoForge entram no escopo inicial ou ficam para depois.
- [ ] **8.4** Seleção de loader e versão do loader na criação de instância.

---

## Fase 9. Mods e modpacks

- [ ] **9.1** Cliente da API do **Modrinth**: buscar mods, ver versões, filtrar por loader e versão do jogo.
- [ ] **9.2** Instalar/atualizar/remover mods numa instância, com verificação de hash e resolução de dependências.
- [ ] **9.3** Importar modpacks `.mrpack`.
- [ ] **9.4** (Opcional) CurseForge, que exige chave de API e termos próprios.
  - 🛑 Cumprimento dos termos de uso das APIs.

---

## Fase 10. Polimento e distribuição

- [ ] **10.1** Instaladores: Windows (NSIS/MSI) e Linux (AppImage + deb; Flatpak como meta).
- [ ] **10.2** Auto-update (updater do Tauri) com assinatura.
  - 🛑 Onde hospedar releases e como gerenciar a chave de assinatura.
- [ ] **10.3** Release automatizado no CI (tag → build → GitHub Release), com o codinome da versão no título.
- [ ] **10.4** Telemetria/crash report: **desligado por padrão**, ou inexistente.
- [ ] **10.5** README, aviso de projeto não oficial, licença, guia de contribuição.
- [ ] **10.6** Revisão de performance: tempo de abertura, RAM ociosa, throughput de download.

### 🏁 Marco 3: v1.0 pública

---

## Ideias futuras (fora do escopo inicial)

- Backup/restauração de mundos
- Importar instâncias de outros launchers (Prism, MultiMC)
- Skins e gerenciamento de capas
- Suporte a macOS
- Plugins/temas
- Servidores favoritos com ping de status

---

## Referências

- Manifesto de versões: `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
- minecraft.wiki (formatos de JSON e autenticação)
- Código de referência: Prism Launcher, HMCL, portablemc, minecraft-launcher-lib
- Tauri v2: https://v2.tauri.app
- The Rust Book (capítulos 4, 6, 9 e 16 são os mais úteis para este projeto)

## Regras permanentes

1. Um item = uma branch = um PR pequeno.
2. Nada de código fora do escopo do item.
3. Toda lógica do core tem testes; nenhum teste depende de rede.
4. Dados do jogo só vêm dos servidores da Mojang; nunca redistribuir arquivos do Minecraft.
5. Sempre perguntar nas decisões marcadas com 🛑.
