# Contributing · Como contribuir

*(Português abaixo.)*

Thanks for wanting to help. Bug reports, ideas and pull requests are welcome.

## Hard rules

1. **Nothing from Mojang/Microsoft (Minecraft) or Re-Logic (Terraria).** No textures (not even
   traced or recoloured), sounds, music, models, fonts or code. Mechanics and ideas are fine;
   files are always ours.
2. **No leaked or decompiled source code**, of any game, ever. Pull requests that look derived
   from it are closed. Work from public sources: wikis, changelogs, open-source projects.
3. **Every third-party asset needs a compatible licence and a credit line.** Accepted: CC0,
   CC BY, CC BY-SA, MIT and similar. Add the source, author and licence to the `CREDITS.md` of
   its folder (see `assets/sounds/*/CREDITS.md`). No "free to download" files without a licence,
   no NC/ND licences.
4. **No resource packs, zips, saves or build outputs in commits.** `.gitignore` already blocks
   the usual ones; check `git status` before committing.
5. **No secrets** (tokens, passwords, keys) in code, config or history.

## Practical notes

- Build and test: `cargo build --release -p game`, `cargo test`.
- Do not run `cargo fmt` over the whole repository: the code keeps its own layout and a mass
  reformat makes every branch conflict. Format only the lines you touch.
- Textures are generated: change the design in `tools/` and run `python3 tools/tex64/build.py`
  instead of editing the 64×64 PNGs by hand. Block models: `python3 tools/gen_block_models.py`.
- Keep sounds low and natural (nothing shrill); new art follows `tools/tex64/README.md`.
- One topic per pull request, with a short description and a screenshot for anything visual.

By contributing you agree that your code is licensed as MIT OR Apache-2.0 and your original
assets as CC BY 4.0 (see [ASSETS-LICENSE.md](ASSETS-LICENSE.md)).

---

## Português

Obrigado por querer ajudar. Relatos de bugs, ideias e pull requests são bem-vindos.

### Regras obrigatórias

1. **Nada da Mojang/Microsoft (Minecraft) nem da Re-Logic (Terraria).** Nem texturas (nem
   decalcadas ou recoloridas), sons, músicas, modelos, fontes ou código. Mecânicas e ideias
   pode; os arquivos são sempre nossos.
2. **Nenhum código vazado ou descompilado**, de jogo nenhum, nunca. PRs que pareçam derivados
   disso serão fechados. Use fontes públicas: wikis, changelogs, projetos de código aberto.
3. **Todo material de terceiros precisa de licença compatível e crédito.** Aceitamos CC0, CC BY,
   CC BY-SA, MIT e parecidas. Escreva fonte, autor e licença no `CREDITS.md` da pasta. Nada de
   arquivo "grátis para baixar" sem licença, nem licenças NC/ND.
4. **Nada de resource packs, zips, mundos salvos ou builds nos commits.** Confira `git status`.
5. **Nenhum segredo** (tokens, senhas, chaves) no código, na configuração ou no histórico.

### Dicas

- Compilar e testar: `cargo build --release -p game`, `cargo test`.
- Não rode `cargo fmt` no repositório inteiro; formate só as linhas que mexer.
- As texturas são geradas: mude o desenho em `tools/` e rode `python3 tools/tex64/build.py`.
- Sons graves e naturais (nada agudo); arte nova segue `tools/tex64/README.md`.
- Um assunto por pull request, com descrição curta e foto se for algo visual.

Ao contribuir, você concorda que seu código fica sob MIT OU Apache-2.0 e seu material original
sob CC BY 4.0 (veja [ASSETS-LICENSE.md](ASSETS-LICENSE.md)).
