# Open PDF

Application bureautique de manipulation de fichiers PDF, 100% locale et open-source. Alternative offline a iLovePDF, construite avec Tauri, Vue 3 et Rust.

## Fonctionnalites

- **Fusionner** — Combiner plusieurs PDF en un seul document (drag & drop pour reordonner)
- **Diviser** — Decoupe par intervalle, selection de pages ou taille maximale
- **Compresser** — Reduire la taille des fichiers PDF
- **Organiser** — Reordonner, supprimer et faire pivoter les pages avec preview
- **Convertir** — Exporter les pages en PNG, JPG ou WEBP
- **Signer** — Ajouter nom, initiales, signature manuscrite, tampon, lieu et date

## Stack technique

| Couche | Technologie |
|--------|-------------|
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust (architecture hexagonale) |
| Framework | Tauri 2 |
| PDF | lopdf |

## Installation

### Prerequis

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.80
- Dependances systeme Tauri ([guide officiel](https://v2.tauri.app/start/prerequisites/))

### Developpement

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

Les binaires sont generes dans `src-tauri/target/release/bundle/` (`.deb`, `.rpm`, `.AppImage` sur Linux, `.msi`/`.exe` sur Windows, `.dmg` sur macOS).

## Licence

MIT
