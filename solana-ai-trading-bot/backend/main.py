
import os
from fastapi import FastAPI, HTTPException
from dotenv import load_dotenv, set_key, dotenv_values
from ai_analysis.gemini_analyzer import get_ai_status
from trading.order_executor import get_trade_history
from blockchain.rpc_client import get_rpc_latency
from backtesting.backtesting_engine import get_tokens_scanned
from trading.wallet_manager import get_tokens_held
from ai_analysis.reputation_db_manager import get_system_health

load_dotenv()

app = FastAPI()

# Endpoint pour lire les variables du .env
@app.get("/api/env")
def get_env_vars():
    env_vars = dotenv_values()
    # Filtrer les clés sensibles si besoin
    return env_vars

# Endpoint pour mettre à jour une variable du .env
@app.post("/api/env/update")
def update_env_var(key: str, value: str):
    env_path = os.path.join(os.path.dirname(__file__), "..", ".env")
    try:
        set_key(env_path, key, value)
        return {"status": "ok", "key": key, "value": value}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Endpoint dashboard pour le frontend
@app.get("/api/dashboard")
def dashboard():
    return {
        "rpc_latency": get_rpc_latency(),
        "tokens_scanned": get_tokens_scanned(),
        "trades_executed": len(get_trade_history()),
        "ai_analysis_status": get_ai_status(),
        "system_health": get_system_health(),
        "tokens_held": get_tokens_held(),
        "trade_history": get_trade_history()
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
    import os
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import StreamingResponse
    import uvicorn
    import io
    import psutil
    import time
    from datetime import datetime, timedelta
    try:
        from dotenv import load_dotenv, set_key, dotenv_values
        load_dotenv()
    except ImportError:
        pass

    app = FastAPI()

    # Endpoint pour lire les variables du .env
    @app.get("/api/env")
    def get_env_vars():
        try:
            env_vars = dotenv_values()
        except Exception:
            env_vars = {}
        return env_vars

    # Endpoint pour mettre à jour une variable du .env
    @app.post("/api/env/update")
    def update_env_var(key: str, value: str):
        env_path = os.path.join(os.path.dirname(__file__), "..", ".env")
        try:
            set_key(env_path, key, value)
            return {"status": "ok", "key": key, "value": value}
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

    from ai_analysis.gemini_analyzer import get_ai_status
    from trading.order_executor import get_trade_history
    from blockchain.rpc_client import get_rpc_latency
    from backtesting.backtesting_engine import get_tokens_scanned
    from trading.wallet_manager import get_tokens_held
    from ai_analysis.reputation_db_manager import get_system_health

    # Endpoint dashboard pour le frontend
    @app.get("/api/dashboard")
    def dashboard():
        return {
            "rpc_latency": get_rpc_latency(),
            "tokens_scanned": get_tokens_scanned(),
            "trades_executed": len(get_trade_history()),
            "ai_analysis_status": get_ai_status(),
            "system_health": get_system_health(),
            "tokens_held": get_tokens_held(),
            "trade_history": get_trade_history()
        }

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
        cpu = psutil.cpu_percent(interval=0.2)
        mem = psutil.virtual_memory().used // 1024 // 1024
        # Optimize latency by reducing the number of API calls
        return {
            "trades_last_minute": trades_last_minute,
            "critical_errors_24h": critical_errors_24h,
            "uptime": uptime,
            "cpu_usage": cpu,
            "memory_usage": mem,
        }

    @app.get("/api/monitoring/metrics/export")
    async def export_monitoring_metrics():
        now = time.time()
        trades_last_minute = len([t for t in _monitoring_trades if now - t < 60])
        critical_errors_24h = len([e for e in _monitoring_last_errors if now - e < 86400])
        uptime = str(timedelta(seconds=int(now - _monitoring_start)))
        cpu = psutil.cpu_percent(interval=0.2)
        mem = psutil.virtual_memory().used // 1024 // 1024
        # Génère le CSV
        output = io.StringIO()
        headers = [
            "trades_last_minute","critical_errors_24h","uptime","cpu_usage","memory_usage"
        ]
        output.write(",".join(headers) + "\n")
        row = [
            str(trades_last_minute),
            str(critical_errors_24h),
            uptime,
            str(cpu),
            str(mem),
        ]
        output.write(",".join(row) + "\n")
        output.seek(0)
        return StreamingResponse(output, media_type="text/csv", headers={"Content-Disposition": "attachment; filename=monitoring_metrics.csv"})

    def schedule_auto_update():
        def updater():
            while True:
                try:
                    import subprocess
                    subprocess.run(["python", "backend/auto_update.py"], capture_output=True, text=True)
                except Exception:
                    pass
                time.sleep(60*60*24)  # 24h
        t = threading.Thread(target=updater, daemon=True)
        t.start()
    schedule_auto_update()

    @app.get("/api/rugpull/detect")
    async def detect_rugpull():
        return {"status": "ok", "report": "Rapport de détection de rugpull"}
    row = [
        str(trades_last_minute),
        str(api_latency),
        str(wallet_balance),
        str(cpu),
        str(mem),
        str(tokens_watched),
        str(critical_errors_24h),
        uptime,
        health.get("global", "Problème")
    ] + [str(health[k]) for k in health.keys()]
    output.write(",".join(row) + "\n")
    output.seek(0)
    return StreamingResponse(output, media_type="text/csv", headers={"Content-Disposition": "attachment; filename=monitoring_metrics.csv"})

# --- Tâche planifiée : auto-update toutes les 24h ---
import threading
import time
def schedule_auto_update():
    def updater():
        while True:
            try:
                import subprocess
                subprocess.run(["python", "backend/auto_update.py"], capture_output=True, text=True)
            except Exception:
                pass
            time.sleep(60*60*24)  # 24h
    t = threading.Thread(target=updater, daemon=True)
    t.start()
schedule_auto_update()

# --- Amélioration de la détection de rugpull ---
@app.get("/api/rugpull/detect")
async def detect_rugpull():
    # Logique pour détecter les rugpulls
    # Exemple: Vérifier les transactions suspectes, les volumes de trading, etc.
    # Retourner un rapport de détection
    return {"status": "ok", "report": "Rapport de détection de rugpull"}

# ...existing code...
