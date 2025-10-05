
import os
import asyncio
from fastapi import FastAPI, HTTPException
from dotenv import load_dotenv, set_key, dotenv_values
from ai_analysis.gemini_analyzer import get_ai_status
# Import des modules principaux du bot
from fastapi.staticfiles import StaticFiles
from ai_analysis.ai_auto_optimizer import AIAutoOptimizer
from trading.decision_module import DecisionModule
from trading.order_executor import OrderExecutor
from blockchain.new_pair_scanner import NewPairScanner
from config.settings import settings

load_dotenv()

app = FastAPI()


# Endpoint pour lire les variables du .env
@app.get("/api/env")
def get_env_vars():
    env_vars = dotenv_values()
    # Filtrer les clés sensibles pour ne pas les exposer au frontend
    sensitive_keys = ["PRIVATE_KEY", "WEB_PASSWORD"]
    safe_vars = {k: v for k, v in env_vars.items() if k not in sensitive_keys}
    return safe_vars

# Endpoint pour mettre à jour une variable du .env
@app.post("/api/env/update")
def update_env_var(key: str, value: str):
    env_path = os.path.join(os.path.dirname(__file__), "..", ".env")
    try:
        set_key(env_path, key, value)

        # Mise à jour dynamique des modules en cours d'exécution
        if key == 'OPENROUTER_API_KEY':
            if hasattr(ai_optimizer, 'gemini_analyzer') and ai_optimizer.gemini_analyzer:
                ai_optimizer.gemini_analyzer.update_api_key(value)
        elif key == 'OPENROUTER_MODEL':
             if hasattr(ai_optimizer, 'gemini_analyzer') and ai_optimizer.gemini_analyzer:
                ai_optimizer.gemini_analyzer.update_model(value)
        elif key == 'TRUSTWALLET_ADDRESS':
            if hasattr(decision_module, 'set_param'):
                decision_module.set_param('trustwallet_address', value)
        elif key == 'BUY_AMOUNT_SOL':
            if hasattr(decision_module, 'set_param'):
                decision_module.set_param('buy_amount_sol', float(value))
        elif key == 'SELL_MULTIPLIER':
            if hasattr(decision_module, 'set_param'):
                decision_module.set_param('sell_multiplier', float(value))

        return {"status": "ok", "key": key, "value": value}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Endpoint dashboard pour le frontend
@app.get("/api/dashboard")
def dashboard():
    # Ce dashboard est maintenant principalement géré par le dashboard de simulation.
    # On peut y mettre des métriques globales du bot.
    return {
        "status": "running",
        "simulation_mode": settings.SIMULATION_MODE,
        "ai_optimizer_status": ai_optimizer.get_status_dict()
    }


@app.get("/api/ai/optimizer/status")
def get_ai_optimizer_status():
    profile_name = ai_optimizer.strategy_profiles[ai_optimizer.current_profile]['name']
    return {
        "current_profile_name": profile_name,
        **ai_optimizer.get_status_dict() # Assurez-vous que cette méthode existe
    }

@app.get("/api/ai/optimizer/history")
def get_ai_optimizer_history():
    """Retourne l'historique des performances de l'optimiseur IA."""
    return ai_optimizer.get_history()

@app.get("/api/pre-flight-check")
async def pre_flight_check():
    """Effectue une série de vérifications avant de lancer le trading réel."""
    if settings.SIMULATION_MODE:
        return {"simulation_mode": True}

    # 1. Vérification du Wallet
    wallet_status = await order_executor.get_wallet_status()

    # 2. Vérification de la connexion RPC
    rpc_ok = False
    try:
        await order_executor.async_client.get_slot()
        rpc_ok = True
    except Exception:
        rpc_ok = False

    # 3. Vérification de l'API Jupiter
    jupiter_ok = await order_executor.check_jupiter_api()

    return {
        "simulation_mode": False,
        "wallet": wallet_status,
        "rpc_connection": rpc_ok,
        "jupiter_api": jupiter_ok,
    }

@app.get("/api/simulation/dashboard")
def simulation_dashboard():
    if not decision_module.simulation_mode:
        raise HTTPException(status_code=400, detail="Le bot n'est pas en mode simulation.")
    
    return {
        "profit_loss_sol": decision_module.get_simulation_profit_loss(),
        "total_trades": len(decision_module.simulation_results),
        "held_tokens_count": len(decision_module.held_tokens),
        "held_tokens_details": decision_module.held_tokens,
        "trade_history": decision_module.simulation_results
    }
from fastapi.responses import StreamingResponse
import io
# --- Endpoint monitoring avancé ---
import psutil
import time
from datetime import datetime, timedelta
_monitoring_start = time.time()
_monitoring_last_errors = []
_monitoring_trades = []

@app.post("/api/monitoring/log_trade")
async def log_trade_monitoring():
    _monitoring_trades.append(time.time())
    return {"status": "ok"}

@app.post("/api/monitoring/log_error")
async def log_error_monitoring():
    _monitoring_last_errors.append(time.time())
    return {"status": "ok"}

@app.get("/api/monitoring/metrics")
async def get_monitoring_metrics():
    now = time.time()
    trades_last_minute = len([t for t in _monitoring_trades if now - t < 60])
    critical_errors_24h = len([e for e in _monitoring_last_errors if now - e < 86400])
    uptime = str(timedelta(seconds=int(now - _monitoring_start)))
    cpu = psutil.cpu_percent(interval=0.1)
    mem = psutil.virtual_memory().used // 1024 // 1024
    return {
        "trades_last_minute": trades_last_minute,
        "critical_errors_24h": critical_errors_24h,
        "uptime": uptime,
        "cpu_usage": cpu,
        "memory_usage": mem,
    }

# --- Amélioration de la détection de rugpull ---
@app.get("/api/rugpull/detect")
async def detect_rugpull():
    # Logique pour détecter les rugpulls
    # Exemple: Vérifier les transactions suspectes, les volumes de trading, etc.
    # Retourner un rapport de détection
    return {"status": "ok", "report": "Rapport de détection de rugpull"}

# ...existing code...

# --- Initialisation des modules principaux ---
order_executor = OrderExecutor(rpc_url=settings.SOLANA_RPC_URL, private_key=settings.PRIVATE_KEY, simulate=settings.SIMULATION_MODE)
decision_module = DecisionModule(order_executor, settings.BUY_AMOUNT_SOL, settings.SELL_MULTIPLIER, simulation_mode=settings.SIMULATION_MODE)
ai_optimizer = AIAutoOptimizer(decision_module)
pair_scanner = NewPairScanner(websocket_url=settings.SOLANA_WSS_URL, decision_module=decision_module)
try:
    from ai_analysis.gemini_analyzer import GeminiAnalyzer
    decision_module.set_gemini_analyzer(GeminiAnalyzer(api_key=settings.OPENROUTER_API_KEY))
except ImportError:
    pass

# --- Événements de démarrage et d'arrêt ---
@app.on_event("startup")
async def startup_event():
    try:
        logger.info("Starting background tasks...")
        ai_optimizer.start()
        asyncio.create_task(pair_scanner.start())
        logger.info("AI Optimizer and Pair Scanner tasks created.")
    except Exception as e:
        logger.critical(f"Failed to start background tasks: {e}")

@app.on_event("shutdown")
def shutdown_event():
    ai_optimizer.stop()
    pair_scanner.stop()

# --- Servir le frontend React ---
app.mount("/", StaticFiles(directory="frontend/build", html=True), name="static")
