def get_ai_status():
    return {"status": "ok"}
import aiohttp
import requests
import os
import time
import re
from loguru import logger


class GeminiAnalyzer:
    async def analyze_logs(self, log_path: str = "simulation_trades.log", github: bool = False, repo: str = None, github_token: str = None) -> list:
        """
        Analyse les logs de trading (local ou GitHub) et retourne des suggestions IA.
        Si github=True, télécharge le log depuis le repo GitHub.
        """
        logs = []
        if github and repo and github_token:
            api_url = f"https://api.github.com/repos/{repo}/contents/{os.path.basename(log_path)}"
            headers = {"Authorization": f"token {github_token}", "Accept": "application/vnd.github.v3.raw"}
            r = requests.get(api_url, headers=headers)
            if r.status_code == 200:
                content = r.text
                logs = content.splitlines()
            else:
                logger.error(f"Impossible de télécharger le log depuis GitHub: {r.text}")
        else:
            if os.path.exists(log_path):
                with open(log_path, "r", encoding="utf-8") as f:
                    logs = f.readlines()
            else:
                logger.error(f"Log local {log_path} introuvable.")
        # Analyse IA des logs (exemple: suggestions d'amélioration)
        prompt = f"Voici l'historique des trades:\n" + "\n".join(logs[-20:])
        return await self.get_suggestions(prompt)

    async def get_suggestions(self, prompt: str) -> list:
        """
        Utilise l'IA (OpenRouter/Gemini) pour générer des suggestions stratégiques ou de sécurité à partir d'un prompt.
        Retourne une liste de suggestions (str).
        """
        try:
            headers = {
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json"
            }
            payload = {
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "Tu es un expert IA en trading crypto. Donne 3 suggestions d'amélioration stratégique ou de sécurité, concises, actionnables, en français, sous forme de liste."},
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 256,
                "temperature": 0.2
            }
            async with aiohttp.ClientSession() as session:
                async with session.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=20) as resp:
                    data = await resp.json()
                    if not data.get("choices") or not data["choices"][0].get("message"):
                        logger.error(f"Réponse inattendue de l'API Gemini: {data}")
                        return ["Erreur: Réponse IA invalide."]
                    content = data["choices"][0]["message"]["content"]
                    # Extraction des suggestions (liste ou texte)
                    suggestions = re.findall(r"[-*•]\s*(.+)", content)
                    if not suggestions:
                        suggestions = [s.strip() for s in content.split("\n") if s.strip()]
                    return suggestions[:5]
        except Exception as e:
            logger.error(f"Erreur Gemini get_suggestions: {e}")
            return [f"Erreur IA: {e}"]

    async def find_associated_creator_wallets(self, creator_address: str) -> list:
        """
        Utilise l'IA pour trouver d'autres adresses de wallet associées à un créateur performant.
        """
        prompt = f"L'adresse de wallet Solana '{creator_address}' a créé un token qui a eu beaucoup de succès (profit x2 en moins d'une heure). En analysant la blockchain Solana (transactions passées, déploiements de contrats, liens via des pools de liquidité), trouve d'autres adresses de wallet qui semblent être contrôlées par la même personne ou le même groupe. Liste uniquement les adresses de wallet Solana valides que tu trouves, une par ligne, sans texte supplémentaire."
        try:
            headers = {
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json"
            }
            payload = {
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "Tu es un expert en analyse de blockchain Solana. Ta tâche est de trouver des wallets associés à une adresse donnée."},
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 512,
                "temperature": 0.3
            }
            async with aiohttp.ClientSession() as session:
                async with session.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=45) as resp:
                    data = await resp.json()
                    if not data.get("choices") or not data["choices"][0].get("message"):
                        logger.error(f"Réponse inattendue de l'API Gemini pour la recherche de wallets: {data}")
                        return []
                    
                    content = data["choices"][0]["message"]["content"]
                    # Regex pour extraire les adresses de wallet Solana
                    solana_address_pattern = r"[1-9A-HJ-NP-Za-km-z]{32,44}"
                    found_wallets = re.findall(solana_address_pattern, content)
                    logger.info(f"IA a trouvé {len(found_wallets)} wallets associés à {creator_address}: {found_wallets}")
                    return found_wallets
        except Exception as e:
            logger.error(f"Erreur Gemini find_associated_creator_wallets: {e}")
            return []
    
    async def get_code_fix_for_error(self, error_log: str, backend_url: str, frontend_url: str):
        """
        Analyse un log d'erreur, demande à l'IA un correctif de code et l'applique.
        """
        if not self.agent:
            logger.error("L'agent Gemini n'est pas initialisé, impossible d'appliquer un correctif.")
            return

        file_list = self.agent.list_files()
        prompt = f"""
Le bot de trading autonome a un problème.
Il tourne sur un serveur distant (Linux Ubuntu 24.04, 4 CPU, 24Go RAM).
L'interface web (frontend) est accessible à l'adresse : {frontend_url}
L'API du bot (backend) est accessible à l'adresse : {backend_url}

Le bot a crashé ou ne répond pas. Voici le log d'erreur capturé :
---
{error_log}
---

Voici la liste des fichiers du projet :
{file_list}

Ta tâche est de fournir un correctif sous forme de bloc de code Python.
Le correctif doit identifier le fichier à modifier et le contenu à remplacer.
Réponds UNIQUEMENT avec le bloc de code Python. Ne fournis aucune explication.
Le code doit utiliser les fonctions de l'agent : `agent.read_file(path)` et `agent.write_file(path, new_content)`.

Exemple de réponse attendue :
```python
file_to_patch = 'backend/trading/decision_module.py'
content = agent.read_file(file_to_patch)
new_content = content.replace('buggy_line()', 'fixed_line()')
agent.write_file(file_to_patch, new_content)
print(f"Fichier {{file_to_patch}} patché.")
```
"""
        try:
            fix_code = await self._get_ia_response(prompt, "Tu es un ingénieur logiciel expert en réparation de code Python.")
            
            # Extraire et exécuter le code Python du bloc markdown
            match = re.search(r"```python\n(.*)\n```", fix_code, re.DOTALL)
            if match:
                code_to_execute = match.group(1)
                logger.critical(f"!!! SECURITY WARNING !!! Exécution de code généré par l'IA. Validez ce code : \n{code_to_execute}")
                logger.info("Code de réparation reçu de l'IA. Exécution...")
                # Créer un scope pour l'exécution avec l'agent disponible
                exec_scope = {'agent': self.agent}
                exec(code_to_execute, exec_scope)
                logger.success("Correctif appliqué avec succès.")
            else:
                logger.error("L'IA n'a pas retourné un bloc de code Python valide pour le correctif.")

        except Exception as e:
            logger.error(f"Erreur lors de l'application du correctif IA : {e}")

    def __init__(self, api_key: str, model: str = None, reputation_db_manager=None, agent=None):
        self.api_key = api_key or os.getenv("OPENROUTER_API_KEY", "")
        self.model = model or os.getenv("OPENROUTER_MODEL", "openai/gpt-3.5-turbo")
        self.reputation_db_manager = reputation_db_manager
        self.agent = agent # L'agent est nécessaire pour modifier les fichiers
        self.recent_logs = []

    def update_api_key(self, new_key: str):
        self.api_key = new_key
        logger.info("OpenRouter API key updated.")

    def update_model(self, new_model: str):
        self.model = new_model
        logger.info(f"OpenRouter model updated: {new_model}")

    async def _get_ia_response(self, prompt: str, system_message: str) -> str:
        """Fonction générique pour interroger l'API OpenRouter."""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        payload = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_message},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 2048,
            "temperature": 0.1
        }
        async with aiohttp.ClientSession() as session:
            async with session.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=120) as resp:
                data = await resp.json()
                if not data.get("choices") or not data["choices"][0].get("message"):
                    logger.error(f"Réponse inattendue de l'API IA: {data}")
                    return f"Erreur: Réponse IA invalide: {data}"
                content = data["choices"][0]["message"]["content"]
                return content

    async def analyze_token(self, token_data: dict) -> float:
        """
        Analyse un token via OpenRouter (GPT, Gemini, Claude, etc.) et retourne un score de risque (0-1).
        """
        prompt = self._build_prompt(token_data)
        try:
            headers = {
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json"
            }
            payload = {
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "Tu es un expert en détection de scam et d'opportunités sur Solana. Donne un score de risque entre 0 (sûr) et 1 (dangereux) pour ce token."},
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 64,
                "temperature": 0.2
            }
            async with aiohttp.ClientSession() as session:
                async with session.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=15) as resp:
                    data = await resp.json()
                    content = data["choices"][0]["message"]["content"]
                    # Extraction du score de risque depuis la réponse IA
                    match = re.search(r"([0-1](?:\.\d+)?)", content)
                    risk_score = float(match.group(1)) if match else 0.5
        except Exception as e:
            logger.error(f"Erreur appel OpenRouter IA : {e}")
            risk_score = 0.5
        comportement = f"AI analyzed, score: {risk_score}"
        wallet_id = token_data.get("mint_address")
        ip_publique = token_data.get("ip_publique")
        tags = token_data.get("tags")
        self.reputation_db_manager.add_entry(wallet_id, ip_publique, tags, comportement, risk_score)
        log_entry = {
            "token": wallet_id,
            "risk_score": risk_score,
            "timestamp": time.time()
        }
        self.recent_logs.append(log_entry)
        if len(self.recent_logs) > 100:
            self.recent_logs.pop(0)
        return risk_score

    def _build_prompt(self, token_data: dict) -> str:
        # Construit un prompt compact pour l'IA
        fields = [f"{k}: {v}" for k, v in token_data.items() if v is not None]
        return "\n".join(fields)
    def export_simulation_report(self, filename: str = "gemini_simulation_report.json"):
        import json
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(self.recent_logs, f, ensure_ascii=False, indent=2)

    def get_recent_logs(self):
        return self.recent_logs