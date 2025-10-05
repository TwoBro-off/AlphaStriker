
import os
import subprocess
import sys
import platform

# --- Configuration ---
BACKEND_DIR = "backend"
FRONTEND_DIR = "frontend"
VENV_DIR = os.path.join(os.path.dirname(__file__), "..", ".venv")
REQUIREMENTS_FILE = os.path.join(BACKEND_DIR, "requirements.txt")
PACKAGE_JSON_FILE = os.path.join(FRONTEND_DIR, "package.json")
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
    UNDERLINE = '\033[4m'

def print_header(message):
    print(f"{colors.HEADER}{colors.BOLD}--- {message} ---"{colors.ENDC})

def print_success(message):
    print(f"{colors.OKGREEN}✓ {message}{colors.ENDC}")

def print_warning(message):
    print(f"{colors.WARNING}! {message}{colors.ENDC}")

def print_fail(message):
    print(f"{colors.FAIL}✗ {message}{colors.ENDC}")
    sys.exit(1)

def run_command(command, cwd=".", error_message="Command failed"):
    """Runs a shell command and exits on failure."""
    try:
        subprocess.run(command, check=True, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return True
    except subprocess.CalledProcessError as e:
        print_fail(f"{error_message}\n{e.stderr.decode('utf-8')}")
        return False

def check_prerequisites():
    """Checks for essential tools like Python, Pip, Node, and Yarn."""
    print_header("1. Checking Prerequisites")
    
    # Check Python
    if sys.version_info < (3, 9):
        print_fail("Python 3.9+ is required. Please upgrade your Python version.")
    print_success("Python 3.9+ found.")

    # Check Pip
    try:
        import pip
        print_success("Pip is available.")
    except ImportError:
        print_fail("Pip is not installed. Please install pip for your Python version.")

    # Check Node.js and Yarn
    if os.path.exists(PACKAGE_JSON_FILE):
        if not run_command("node -v", error_message="Node.js is not installed."):
            print_fail("Node.js is required for the frontend. Please install it.")
        print_success("Node.js found.")
        
        if not run_command("yarn -v", error_message="Yarn is not installed."):
            print_warning("Yarn is not installed. Attempting to install with npm.")
            if not run_command("npm install -g yarn", error_message="Failed to install Yarn."):
                print_fail("Could not install Yarn. Please install it manually.")
        print_success("Yarn found (or installed).")

def setup_api_key():
    """Prompts the user for their Gemini API key and saves it to a .env file."""
    print_header("2. Configuring Gemini API Key")
    if os.path.exists(ENV_FILE):
        print_warning(f"{ENV_FILE} already exists. Checking for GEMINI_API_KEY...")
        with open(ENV_FILE, 'r') as f:
            if 'GEMINI_API_KEY' in f.read():
                print_success("GEMINI_API_KEY already set in .env file.")
                return
    
    gemini_api_key = input(f"{colors.OKCYAN}Please enter your Gemini API Key: {colors.ENDC}").strip()
    if not gemini_api_key:
        print_fail("Gemini API Key cannot be empty.")
    
    with open(ENV_FILE, "a") as f:
        f.write(f"\nGEMINI_API_KEY={gemini_api_key}\n")
    print_success(f"Gemini API Key saved to {ENV_FILE}")

def install_dependencies():
    """Installs backend and frontend dependencies."""
    print_header("3. Installing Dependencies")

    # Backend dependencies
    print(f"{colors.OKBLUE}Creating Python virtual environment in '{VENV_DIR}'...{colors.ENDC}")
    if not os.path.exists(VENV_DIR):
        run_command(f"{sys.executable} -m venv {VENV_DIR}", error_message="Failed to create virtual environment.")
    print_success("Virtual environment created.")

    pip_executable = os.path.join(VENV_DIR, 'Scripts' if platform.system() == "Windows" else 'bin', 'pip')
    
    print(f"{colors.OKBLUE}Installing backend dependencies from {REQUIREMENTS_FILE}...{colors.ENDC}")
    run_command(f"{pip_executable} install -r {REQUIREMENTS_FILE}", error_message="Failed to install backend dependencies.")
    print_success("Backend dependencies installed.")

    # Frontend dependencies
    if os.path.exists(PACKAGE_JSON_FILE):
        print(f"{colors.OKBLUE}Installing frontend dependencies from {PACKAGE_JSON_FILE}...{colors.ENDC}")
        run_command("yarn install", cwd=FRONTEND_DIR, error_message="Failed to install frontend dependencies.")
        print_success("Frontend dependencies installed.")
    else:
        print_warning("package.json not found. Skipping frontend dependency installation.")

def self_diagnostics():
    """Runs basic diagnostics to ensure setup was successful."""
    print_header("4. Running Self-Diagnostics")
    
    # Verify Gemini API key
    print(f"{colors.OKBLUE}Verifying Gemini API connectivity...{colors.ENDC}")
    python_executable = os.path.join(VENV_DIR, 'Scripts' if platform.system() == "Windows" else 'bin', 'python')
    verification_script = '''
import os
from dotenv import load_dotenv
import google.generativeai as genai
load_dotenv()
api_key = os.getenv("GEMINI_API_KEY")
if not api_key:
    print("ERROR: GEMINI_API_KEY not found in .env file.")
    sys.exit(1)
try:
    genai.configure(api_key=api_key)
    model = genai.GenerativeModel('gemini-pro')
    model.generate_content("test")
    print("SUCCESS: Gemini API key is valid and working.")
except Exception as e:
    print(f"ERROR: Gemini API key is invalid or there is a connection issue.\n{e}")
    sys.exit(1)
'''
    try:
        result = subprocess.run(
            [python_executable, "-c", verification_script],
            check=True, capture_output=True, text=True
        )
        if "SUCCESS" in result.stdout:
            print_success("Gemini API verification successful.")
        else:
            print_fail(f"Gemini API verification failed:\n{result.stdout}")
    except subprocess.CalledProcessError as e:
        print_fail(f"Gemini API verification script failed to run:\n{e.stderr}")

def main():
    """Main function to run the setup process."""
    print_header("Starting AlphaStriker AI Bot Automated Setup")
    print(f"{colors.WARNING}This script will guide you through the setup process.{colors.ENDC}")
    
    check_prerequisites()
    setup_api_key()
    install_dependencies()
    self_diagnostics()
    
    print_header("🎉 Setup Complete! 🎉")
    print_success("All steps completed successfully.")
    print(f"{colors.OKCYAN}To start the bot, activate the virtual environment and run the main application:{colors.ENDC}")
    if platform.system() == "Windows":
        print(f"  {colors.BOLD}cd c:\\Users\\pc\\solana-ai-trading-bot && .\\.venv\\Scripts\\activate && python -m solana-ai-trading-bot.backend.main{colors.ENDC}")
    else:
        print(f"  {colors.BOLD}cd /path/to/solana-ai-trading-bot && source ./.venv/bin/activate && python -m solana-ai-trading-bot.backend.main{colors.ENDC}")

if __name__ == "__main__":
    main()
