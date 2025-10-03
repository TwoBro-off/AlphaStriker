def get_ai_status():
    return {"status": "ok"}
import aiohttp
import os
import time
from loguru import logger


class GeminiAnalyzer:
    async def analyze_logs(self, log_path: str = "simulation_trades.log", github: bool = False, repo: str = None, github_token: str = None) -> list:
        """
        Analyse les logs de trading (local ou GitHub) et retourne des suggestions IA.
        Si github=True, télécharge le log depuis le repo GitHub.
        """
        logs = []
        if github and repo and github_token:
            import requests
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
                    content = data["choices"][0]["message"]["content"]
                    # Extraction des suggestions (liste ou texte)
                    import re
                    suggestions = re.findall(r"[-*•]\s*(.+)", content)
                    if not suggestions:
                        suggestions = [s.strip() for s in content.split("\n") if s.strip()]
                    return suggestions[:5]
        except Exception as e:
            logger.error(f"Erreur Gemini get_suggestions: {e}")
            return [f"Erreur IA: {e}"]
    def __init__(self, api_key: str, reputation_db_manager, model: str = None):
        self.api_key = api_key or os.getenv("OPENROUTER_API_KEY", "")
        self.model = model or os.getenv("OPENROUTER_MODEL", "openai/gpt-3.5-turbo")
        self.reputation_db_manager = reputation_db_manager
        self.recent_logs = []

    def update_api_key(self, new_key: str):
        self.api_key = new_key
        logger.info("OpenRouter API key updated.")

    def update_model(self, new_model: str):
        self.model = new_model
        logger.info(f"OpenRouter model updated: {new_model}")

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
                    import re
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
        if token_data.get('liquidity', 0) < 3.0:
            score -= 0.3
        if token_data.get('honeypot', False):
            score -= 0.4
        if token_data.get('blacklisted', False):
            score -= 0.5
        return max(0.0, min(1.0, score))
    def export_simulation_report(self, filename: str = "gemini_simulation_report.json"):
        import json
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(self.recent_logs, f, ensure_ascii=False, indent=2)

    def get_recent_logs(self):
        return self.recent_logs