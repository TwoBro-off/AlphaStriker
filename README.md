---
<div align="center">
  <img src="frontend/public/logo192.png" width="96" alt="AlphaStriker Logo" />
  <h1>AlphaStriker – Solana AI Trading Bot</h1>
  <b>Bot de trading IA autonome pour Solana, avec auto-réparation et déploiement simplifié.</b>
</div>

---

## 🚀 Fonctionnalités Principales

- **Déploiement 100% Autonome** : Une seule commande lance un agent IA qui installe, compile, démarre, surveille et répare le bot.
- **Agent IA Auto-Réparateur** : Utilise Gemini pour analyser les erreurs, écrire des correctifs de code et les appliquer en temps réel.
- **Trading Haute Fréquence** : Analyse et achète de nouveaux tokens en moins de 500ms grâce à des vérifications parallélisées.
- **Sécurité Renforcée** : Vérifications systématiques de liquidité, honeypot, taxes, et sellability via des APIs externes avant chaque achat.
- **Dashboard Complet** : Interface en React pour le suivi des performances, la configuration des paramètres et la visualisation des trades.
- **Optimisation Continue** : L'IA analyse les performances et ajuste dynamiquement les stratégies de trading.

---

## 🚦 Installation et Lancement via l'Agent IA

L'installation est entièrement gérée par un agent IA. Une seule commande suffit après avoir cloné le projet.

### 1. Prérequis (à installer manuellement)
```bash
# Sur Ubuntu/Debian
sudo apt update && sudo apt install -y git python3-venv nodejs npm
# Assurez-vous d'avoir Python 3.9+ et Node.js 18+ sur votre système.
```
# Cloner le projet
git clone https://github.com/votre-utilisateur/solana-ai-trading-bot.git
cd solana-ai-trading-bot

-# Installer les dépendances de l'agent IA
-python3 -m venv venv
-source venv/bin/activate
-pip install -r backend/requirements.txt
-```
-
-### 3. Configuration
-```bash
-# Copier le fichier d'exemple
-cp .env.example .env
-
-# Éditer le fichier pour ajouter vos clés
-nano .env
-```
# Lancer le script de configuration et de démarrage autonome
python3 run_ai_setup.py
+```

-
-
-- Dashboard: http://<YOUR_SERVER_IP>:3000
-- Backend API: http://<YOUR_SERVER_IP>:8000
+L'agent IA va maintenant :
+1.  **Vous demander votre clé API si elle est manquante.**
+2.  Installer toutes les dépendances (backend et frontend).
+3.  Compiler l'interface web.
+4.  Lancer le serveur du bot.
+5.  Surveiller le bot, le redémarrer et tenter de le réparer automatiquement en cas de crash.

---


# 📊 Dashboard & Features

- **Dashboard**: Real-time metrics, trades, logs, AI chat, advanced monitoring
- **Settings**: All secrets/keys editable in UI (never exposed to frontend code)
- **Gemini Chat**: Ask Gemini to analyze, explain, or patch code (Python/React)
- **Backtesting**: Simulate strategies before live trading
- **Security**: All trades pass liquidity, honeypot, blacklist, holders checks
- **Logs**: simulation_trades.log, real_trades.log (auto-generated)
- **Self-repair**: Gemini agent can patch/restore backend code if broken

---

# 🔒 Security & AI

- All secrets in `.env` (never commit or share)
- Gemini AI can only patch non-secret files (user must validate all changes)
- All code changes via Gemini are logged and auditable
- Backend is always the source of truth for parameters

---

# 🤖 Gemini AI Integration

- Gemini (OpenRouter) can read/patch any non-secret file for self-repair
- All requests/responses are logged for audit
- User must validate all code changes before application
- Gemini agent: `backend/ai_analysis/gemini_agent.py` (never delete/corrupt)

---

# 📝 License

MIT