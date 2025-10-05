<div align="center">
  <img src="frontend/public/logo192.png" width="96" alt="AlphaStriker Logo" />
  <h1>AlphaStriker – Guide d'Installation Autonome (ARM / Oracle Cloud)</h1>
  <b>Bot de trading ultra-robuste pour l'architecture ARM (Ampere A1 sur Oracle Cloud), combinant FastAPI, React, et l'IA Gemini.</b>
</div>

---
## 🚀 Fonctionnalités Principales
- **Sécurité renforcée** : vérifications de liquidité, honeypot, blacklist, et distribution des détenteurs.
- **Base de données de réputation locale** (SQLite) pour les tokens et les wallets.
- **Dashboard avancé** (React/Tailwind) avec métriques en direct, logs, et chat avec l'IA.
- **Déploiement Autonome** : L'IA gère la compilation, le démarrage, la surveillance et la réparation du bot.
- **Agent IA auto-réparateur** : Gemini peut corriger le code via le dashboard.
---
## 🚦 Guide d'Installation

Ce guide est optimisé pour la meilleure performance et le meilleur rapport coût/efficacité sur les instances **Ampere A1 (ARM)** d'Oracle Cloud.

### 1. Prérequis

Assurez-vous que les logiciels suivants sont installés sur votre serveur Ubuntu 22.04/24.04 LTS.

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y git python3-pip python3.10-venv nodejs npm
```

### 2. Cloner le Dépôt

```bash
git clone https://github.com/votre-utilisateur/solana-ai-trading-bot.git
cd solana-ai-trading-bot
```

### 3. Installer les dépendances (Docker, Node.js, etc.)

Le script suivant installe tout ce dont vous avez besoin.

```bash
sudo apt install -y git curl python3 python3-pip ufw

# Installer Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
newgrp docker
docker --version

# Installer Node.js (v18 LTS)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Installer Yarn
npm install -g yarn
yarn --version
```

### 4. Cloner le dépôt et lancer l'installation

```bash
git clone https://github.com/votre-utilisateur/solana-ai-trading-bot.git
cd solana-ai-trading-bot

# Rendre le script exécutable et le lancer
chmod +x install.sh
./install.sh
```

Le script `install.sh` va automatiquement :
- Détecter l'architecture ARM.
- Construire l'image Docker optimisée.
- Créer le fichier `.env` pour vos secrets.

### 5. Configurer vos secrets

Éditez le fichier `.env` qui vient d'être créé pour y ajouter vos clés API et votre clé privée de wallet.

```bash
nano .env
```

Remplissez les champs suivants :
- `PRIVATE_KEY` : La clé privée de votre **wallet de trading dédié**.
- `OPENROUTER_API_KEY` : Votre clé API OpenRouter pour l'IA.
- `TRUSTWALLET_ADDRESS` : L'adresse publique de votre wallet de sécurité.

### 6. Lancer le Bot

```bash
./run.sh
```

### 7. Accéder au Dashboard

Une fois le bot lancé, vous pouvez accéder à l'interface :

- **Dashboard** : `http://<IP_DE_VOTRE_SERVEUR>:3000`
- **API Backend** : `http://<IP_DE_VOTRE_SERVEUR>:8000/docs`

**Conseils :**
- Pour mettre à jour le bot : `git pull && ./install.sh && ./run.sh`
- Pour voir les logs en direct : `docker logs -f alphastriker-instance`
