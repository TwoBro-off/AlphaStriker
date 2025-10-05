import asyncio
import time
import os
from dotenv import load_dotenv
from loguru import logger

# Charger les variables d'environnement en premier
load_dotenv()

from backend.ai_analysis.gemini_agent import GeminiAgent
from backend.ai_analysis.gemini_analyzer import GeminiAnalyzer
from backend.ai_analysis.ai_launcher import AILauncher
from backend.config.settings import settings

async def main_ai_loop():
    """
    Boucle principale de l'agent IA.
    L'IA tente de lancer le bot, et si elle échoue, elle tente de le réparer.
    """
    logger.info("🤖 Démarrage du Lanceur d'Agent IA Autonome...")

    # Initialisation des modules clés pour l'IA
    launcher = AILauncher(command=["uvicorn", "backend.main:app", "--host", "0.0.0.0", "--port", "8000"])
    agent = GeminiAgent(root_dir=".")
    # L'analyzer a besoin de l'agent pour pouvoir modifier les fichiers
    analyzer = GeminiAnalyzer(api_key=settings.OPENROUTER_API_KEY, model=settings.GEMINI_MODEL, agent=agent)

    # --- Séquence de démarrage autonome ---
    logger.info("🔧 Lancement de la séquence de démarrage autonome...")

    # 1. Installer les dépendances du frontend
    logger.info("... Installation des dépendances du frontend (yarn install)...")
    frontend_install_result = agent.execute_command("cd frontend && yarn install")
    if frontend_install_result["returncode"] != 0:
        logger.error(f"Échec de l'installation des dépendances frontend: {frontend_install_result['stderr']}")
        # L'IA pourrait tenter de réparer ici

    # 2. Compiler le frontend
    logger.info("... Compilation du frontend (yarn build)...")
    frontend_build_result = agent.execute_command("cd frontend && yarn build")
    if frontend_build_result["returncode"] != 0:
        logger.error(f"Échec de la compilation du frontend: {frontend_build_result['stderr']}")
        # L'IA pourrait tenter de réparer ici

    logger.success("✅ Séquence de démarrage autonome terminée.")

    # Récupérer les URLs depuis les settings
    backend_url = settings.BACKEND_URL
    frontend_url = settings.FRONTEND_URL
    health_check_endpoint = f"{backend_url}/api/dashboard" # Utilise un endpoint existant pour le health check

    while True:
        # Vérification de santé plus robuste : le processus tourne ET l'API répond
        backend_healthy = launcher.is_running() and await launcher.check_endpoint_health(health_check_endpoint)

        if not backend_healthy:
            logger.warning("Le bot n'est pas en cours d'exécution. Tentative de démarrage...")
            launcher.start_bot()
            await asyncio.sleep(10) # Laisser le temps au bot de démarrer ou de crasher

            # Nouvelle vérification après la tentative de démarrage
            if not (launcher.is_running() and await launcher.check_endpoint_health(health_check_endpoint)):
                logger.error("Échec du démarrage du bot. Analyse des erreurs...")
                error_logs = launcher.get_error_logs()

                if error_logs:
                    logger.info("Erreurs détectées. Demande de correctif à l'IA...")
                    print("--- Début des logs d'erreur ---")
                    print(error_logs)
                    print("--- Fin des logs d'erreur ---")

                    # L'IA analyse l'erreur et tente d'appliquer un correctif
                    await analyzer.get_code_fix_for_error(error_logs, backend_url, frontend_url)

                    # Après la tentative de réparation, on attend avant de relancer
                    logger.info("Tentative de réparation effectuée. Le système va redémarrer dans 30 secondes.")
                    await asyncio.sleep(30)
                else:
                    logger.info("Aucun log d'erreur spécifique trouvé. Le bot a peut-être planté sans message.")
                    await asyncio.sleep(30) # Attendre avant de réessayer
            else:
                logger.success("✅ Le bot a démarré avec succès et est en cours d'exécution.")

        # Vérification toutes les 60 secondes
        await asyncio.sleep(60)

if __name__ == "__main__":
    try:
        asyncio.run(main_ai_loop())
    except KeyboardInterrupt:
        logger.info("Arrêt du Lanceur d'Agent IA.")