# Licht Launcher (Tauri v2)

## Contexto
- Launcher **não oficial** de Minecraft: Java Edition, multiplataforma (**Windows e Linux**), feito com **Tauri v2**.
- Frontend: **React + TypeScript + Vite**, **Tailwind CSS**, **shadcn/ui** (Radix UI), **Lucide** (ícones), **TanStack Query** (chamadas `invoke`), **Zustand** (estado global), **TanStack Virtual** (listas grandes). Backend: Rust.
- Nome exibido: **Licht Launcher**. Repositório/pacotes: `licht-launcher`. Binário/CLI: `licht`. Crate do core: `licht-core`.
- O dono do projeto é desenvolvedor sênior em TypeScript/Node, mas tem **POUCO conhecimento de Rust**.
- O plano completo está em `ROADMAP.md`. Trabalhe sempre em **um item por vez**.

## Como trabalhar comigo
- Antes de qualquer código, apresente um **PLANO curto** (arquivos que serão criados/alterados, abordagem, alternativas) e espere minha aprovação.
- **Pergunte** quando houver decisão: nova dependência, mudança de arquitetura, API pública entre Rust e frontend, segurança/autenticação, qualquer ambiguidade. Os itens marcados com 🛑 no roadmap sempre exigem minha aprovação. Não decida sozinho.
- Uma tarefa = uma branch = um PR pequeno e focado. Nunca misture assuntos.
- **Não escreva código além do que a tarefa pede.** Sem abstrações "para o futuro", sem features extras, sem refatorar o que não foi pedido.
- Siga a arquitetura abaixo. Se achar que ela deve mudar, **proponha antes**, não mude.
- Edições de arquivo dentro de um plano já aprovado não precisam de confirmação individual.
- Se algo na tarefa conflitar com este arquivo ou com o roadmap, pare e me avise.

## Idioma
- Converse comigo em **português**.
- Código, nomes de identificadores, comentários e mensagens de commit em **inglês** (Conventional Commits: `feat:`, `fix:`, `test:`, `chore:`...).
- Descrições de PR em **português**.
- *(Ajuste esta seção se preferir outro padrão.)*

## Arquitetura
- Cargo workspace:
  - `crates/licht-core`: toda a lógica (manifestos, downloads, regras, Java, launch, auth). **Sem dependência do Tauri.** Testável sozinho.
  - `src-tauri`: camada fina. Apenas `#[tauri::command]` e eventos que chamam o `licht-core`. **Sem regra de negócio aqui.**
  - `src/`: frontend. Sem lógica de Minecraft; só UI e chamadas `invoke`. Estrutura: `src/components/ui` (shadcn/ui), `src/features/*` (telas e lógica de UI por funcionalidade), `src/lib` (utilitários e wrappers de `invoke`).
- Comunicação Rust → frontend: **eventos** (ex.: progresso de download). Frontend → Rust: **commands tipados**.
- Tipos compartilhados entre Rust e TS devem ser **gerados** (`ts-rs` ou `specta`, a definir), nunca duplicados à mão.
- Erros: `thiserror` no core, `anyhow` apenas nas bordas. **Nada de `unwrap()`** fora de testes.
- Logs com `tracing`.
- Use **Tauri v2**. Não use APIs ou exemplos da v1.

## UI e design system (consistência acima de criatividade)
- Use **somente** os componentes de `src/components/ui` e os **tokens do tema** (variáveis CSS). Se faltar um componente, proponha adicioná-lo ao design system em vez de criar um estilo avulso.
- **Proibido:** estilos inline, cores/tamanhos "soltos" (ex.: `#3b82f6`, `w-[137px]`) fora dos tokens, e qualquer biblioteca de UI nova sem minha aprovação.
- Antes de criar uma tela, **liste os componentes existentes** que serão usados e quais faltam.
- Acessibilidade: tudo operável por teclado, com foco visível e rótulos corretos (o Radix já ajuda; não remova).
- O frontend roda em **WebView2 (Windows)** e **WebKitGTK (Linux)**. Evite `backdrop-filter`, blurs, sombras grandes e animações complexas. Se usar um recurso moderno de CSS, avise para eu testar nos dois sistemas.
- Chamadas ao Rust via TanStack Query (cache, loading e erro). Eventos de progresso com throttling (~100 ms). Listas longas sempre virtualizadas.
- Tipos compartilhados com o Rust são gerados; nunca redeclare à mão.

## Rust (nível iniciante: priorize código simples)
- Prefira clareza a "idiomático avançado". Evite lifetimes complexos, macros próprias e `unsafe`. Se for inevitável, explique e peça aprovação.
- Use `clone()` sem culpa quando simplificar.
- Em todo PR, inclua na descrição a seção **"Conceitos de Rust usados"**, explicando em linguagem simples (com comparação com TS/Node quando fizer sentido) qualquer conceito novo: ownership, borrowing, `Result`, traits, async/tokio.
- Quando houver erro do compilador, **explique a causa** antes de corrigir.
- Comente apenas o "porquê" não óbvio.

## Qualidade (obrigatório antes de abrir PR)
- Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- Frontend: `pnpm lint`, `pnpm typecheck`, `pnpm test` (Vitest + Testing Library; componentes novos com ao menos um teste de renderização/interação)
- Testes automatizados para toda lógica do core. Use **fixtures** (JSONs reais da Mojang salvos em `tests/fixtures`). **Nenhum teste pode depender de rede.**
- Se algum check falhar e você não conseguir resolver, diga no PR em vez de esconder.

## Formato do PR
- Título curto. Descrição com: o que mudou, por quê, como testar, decisões tomadas, conceitos de Rust usados, pontos onde quero minha opinião.

## Regras de domínio
- Dados do jogo vêm **sempre dos servidores da Mojang**. Nunca redistribuir arquivos do Minecraft.
- Verificar **SHA1** de tudo que for baixado.
- Nada de tokens/segredos no repositório ou nos logs. Tokens ficam no **keyring do SO**.
- Caminhos e separadores devem funcionar em Windows e Linux (use `std::path` e a crate `directories`, nunca concatene strings).
- O projeto **não é oficial**: mantenha o aviso "não afiliado à Mojang Studios nem à Microsoft" no README e na tela "Sobre". Não use "Minecraft" ou "Mojang" no nome do projeto, do pacote ou do binário, e não use logos ou arte oficiais.

## Convenções de release
- Cada release `0.x` recebe um **codinome de flor em alemão**, conforme a tabela do `ROADMAP.md` (0.1 Lilie, 0.2 Iris, 0.3 Rose...). O codinome vai na tela "Sobre", no título da release e no nome do instalador.

## Referências úteis
- Manifesto: `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
- minecraft.wiki (formatos de JSON e autenticação)
- Código de referência: Prism Launcher, HMCL, portablemc, minecraft-launcher-lib
- Tauri v2: https://v2.tauri.app
