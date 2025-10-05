import os
import subprocess
import sys
import platform
from dotenv import load_dotenv, find_dotenv, set_key

# --- Configuration ---
BACKEND_DIR = "backend"
FRONTEND_DIR = "frontend"
VENV_DIR = ".venv"
REQUIREMENTS_FILE = os.path.join(BACKEND_DIR, "requirements.txt")
ENV_FILE = ".env"

# --- ANSI Colors for better output ---
class colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def print_header(message):
    print(f"\n{colors.HEADER}{colors.BOLD}--- {message} ---{colors.ENDC}")

def print_success(message):
    print(f"{colors.OKGREEN}✓ {message}{colors.ENDC}")

def print_warning(message):
    print(f"{colors.WARNING}⚠ {message}{colors.ENDC}")

def print_fail(message, details=""):
    print(f"{colors.FAIL}✗ {message}{colors.ENDC}")
    if details:
        print(details)
    sys.exit(1)

def run_command(command, cwd=".", error_message="Command failed", capture=False):
    """Runs a shell command and exits on failure."""
    try:
        if platform.system() == "Windows":
            # shell=True is often needed on Windows for commands like 'npm'
            process = subprocess.run(command, check=True, cwd=cwd, shell=True, capture_output=capture, text=True)
        else:
            process = subprocess.run(command.split(), check=True, cwd=cwd, capture_output=capture, text=True)
        return process
    except subprocess.CalledProcessError as e:
        print_fail(error_message, e.stderr)
    except FileNotFoundError:
        print_fail(f"Command not found: {command.split()[0]}. Is it installed and in your PATH?")

def setup_env_file():
    """Ensures .env file exists and prompts for API key if missing."""
    print_header("1. Configuration de l'environnement")
    if not os.path.exists(ENV_FILE):
        print_warning(f"Le fichier {ENV_FILE} n'existe pas. Copie depuis .env.example...")
        try:
            import shutil
            shutil.copy(".env.example", ENV_FILE)
            print_success(f"{ENV_FILE} créé avec succès.")
        except FileNotFoundError:
            print_fail(".env.example non trouvé. Assurez-vous que le fichier existe.")

    load_dotenv(ENV_FILE)
    if not os.getenv("OPENROUTER_API_KEY"):
        print_warning("La clé API OpenRouter/Gemini est manquante.")
        api_key = input(f"{colors.OKCYAN}Veuillez entrer votre clé API OpenRouter : {colors.ENDC}").strip()
        if not api_key:
            print_fail("La clé API ne peut pas être vide.")
        set_key(ENV_FILE, "OPENROUTER_API_KEY", api_key)
        print_success("Clé API enregistrée dans .env")
    else:
        print_success("Clé API OpenRouter trouvée dans .env")

def install_dependencies():
    """Installs backend and frontend dependencies."""
    print_header("2. Installation des dépendances")

    # Backend
    if not os.path.exists(VENV_DIR):
        print(f"{colors.OKBLUE}Création de l'environnement virtuel Python dans '{VENV_DIR}'...{colors.ENDC}")
        run_command(f"{sys.executable} -m venv {VENV_DIR}", error_message="Échec de la création de l'environnement virtuel.")
        print_success("Environnement virtuel créé.")

    pip_executable = os.path.join(VENV_DIR, 'Scripts' if platform.system() == "Windows" else 'bin', 'pip')
    run_command(f"{pip_executable} install -r {REQUIREMENTS_FILE}", error_message="Échec de l'installation des dépendances backend.")
    print_success("Dépendances backend installées.")

def launch_ai_agent():
    """Launches the main autonomous AI agent."""
    print_header("3. Lancement de l'Agent IA Autonome")
    print(f"{colors.OKCYAN}L'agent IA va maintenant prendre le contrôle pour compiler le frontend et lancer le bot.{colors.ENDC}")
    print(f"{colors.OKCYAN}Le bot sera surveillé et redémarré/réparé si nécessaire.{colors.ENDC}")
    print(f"{colors.WARNING}Pour arrêter l'agent, utilisez Ctrl+C.{colors.ENDC}")

    python_executable = os.path.join(VENV_DIR, 'Scripts' if platform.system() == "Windows" else 'bin', 'python')
    agent_script_path = os.path.join(BACKEND_DIR, "ai_analysis", "run_ai_agent.py")

    try:
        # Utilise Popen pour ne pas bloquer et afficher la sortie en direct
        process = subprocess.Popen([python_executable, agent_script_path], text=True)
        process.wait()
    except KeyboardInterrupt:
        print(f"\n{colors.WARNING}Arrêt manuel de l'agent IA...{colors.ENDC}")
        process.terminate()
    except Exception as e:
        print_fail("Une erreur est survenue lors du lancement de l'agent IA.", str(e))

if __name__ == "__main__":
    print_header("Démarrage de l'installation et du lancement autonome d'AlphaStriker")
    setup_env_file()
    install_dependencies()
    launch_ai_agent()
    print_header("🎉 L'agent IA a terminé ou a été arrêté. 🎉")