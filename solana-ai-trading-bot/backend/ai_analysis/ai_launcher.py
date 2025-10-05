import subprocess
import threading
from typing import List
import aiohttp
from loguru import logger

class AILauncher:
    """
    Gère le lancement du bot principal en tant que sous-processus,
    permettant à l'agent IA de le surveiller, le redémarrer et lire ses logs.
    """
    def __init__(self, command: List[str]):
        self.command = command
        self.process = None
        self.stdout = []
        self.stderr = []

    def _read_stream(self, stream, storage):
        for line in iter(stream.readline, ''):
            decoded_line = line.decode('utf-8').strip()
            logger.info(f"[Bot Process] {decoded_line}")
            storage.append(decoded_line)
        stream.close()

    def start_bot(self):
        if self.is_running():
            logger.warning("Le bot est déjà en cours d'exécution.")
            return

        logger.info(f"Lancement du bot avec la commande: {' '.join(self.command)}")
        self.process = subprocess.Popen(self.command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

        # Démarrer des threads pour lire stdout et stderr sans bloquer
        threading.Thread(target=self._read_stream, args=(self.process.stdout, self.stdout), daemon=True).start()
        threading.Thread(target=self._read_stream, args=(self.process.stderr, self.stderr), daemon=True).start()

    def stop_bot(self):
        if self.is_running():
            logger.info("Arrêt du processus du bot...")
            self.process.terminate()
            self.process = None

    def is_running(self) -> bool:
        return self.process and self.process.poll() is None

    def get_error_logs(self) -> str:
        return "\n".join(self.stderr)

    async def check_endpoint_health(self, url: str) -> bool:
        """Vérifie si une URL est accessible et retourne un code 2xx."""
        if not url:
            return False
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(url, timeout=5) as response:
                    is_healthy = 200 <= response.status < 300
                    logger.info(f"Health check pour {url}: Status {response.status} -> {'Sain' if is_healthy else 'Échec'}")
                    return is_healthy
        except Exception as e:
            logger.warning(f"Health check pour {url} a échoué avec une erreur: {e}")
            return False