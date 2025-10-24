import React, { useState, useEffect, useCallback } from 'react';
import { Line } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler } from 'chart.js';
import ErrorBoundary from './ErrorBoundary.js';
import AIStatusPanel from './AIStatusPanel.js';
import DetailedStats from './DetailedStats.js';
import AIOptimizerStatus from './AIOptimizerStatus.js';
import BotTesting from './BotTesting.js';
import PerformanceMetrics from './PerformanceMetrics.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler);

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';
const OFFLINE_STATUS = { is_running: false, current_mode: 'Déconnecté', is_offline: true };

function DashboardContent() {
  const [botStatus, setBotStatus] = useState(OFFLINE_STATUS);
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isPageLoading, setIsPageLoading] = useState(true);
  const [simulationData, setSimulationData] = useState({ trade_history: [], profit_loss_sol: 0, total_trades: 0, held_tokens_count: 0 });
  const [readiness, setReadiness] = useState({ is_ready: false, checks: {} });
  const [activityLog, setActivityLog] = useState([]);

  const fetchBotStatus = useCallback(async () => {
    try {
      const response = await fetch(`${API_URL}/api/bot/status`);
      if (!response.ok) {
        console.error('Backend error:', response.status);
        throw new Error('Le backend ne répond pas');
      }
      const data = await response.json();
      setBotStatus(data);
      if (error) setError('');
      return data;
    } catch (err) {
      console.error('Failed to fetch bot status:', err);
      setBotStatus(OFFLINE_STATUS);
      setError('Impossible de contacter le backend. Vérifiez que le serveur est en cours d\'exécution.');
      return null;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [error, setBotStatus, setError]); // Correctly include error dependency

  const fetchSimulationData = useCallback(async () => {
    try {
      const response = await fetch(`${API_URL}/api/simulation/dashboard`);
      if (!response.ok) return;
      const data = await response.json();
      setSimulationData(data);
    } catch (err) { console.error("Failed to fetch simulation data:", err); }
  }, []);

  const fetchReadiness = useCallback(async () => {
    if (!botStatus || botStatus.is_offline) return;
    try {
      const response = await fetch(`${API_URL}/api/bot/readiness`);
      if (!response.ok) return;
      const data = await response.json();
      setReadiness(data);
    } catch (err) { /* fail silently */ }
  }, [botStatus]);
  
  const fetchActivityLog = useCallback(async () => {
    if (!botStatus || botStatus.is_offline) return;
    try {
      const response = await fetch(`${API_URL}/api/bot/activity`);
      if (!response.ok) return;
      const data = await response.json();
      setActivityLog(Array.isArray(data) ? data : []);
    } catch (err) { /* fail silently */ }
  }, [botStatus]);

  useEffect(() => {
    // Effet unifié pour gérer tout le cycle de vie du polling de données.
    const updateData = async () => {
      // 1. Toujours récupérer le statut le plus récent
      const status = await fetchBotStatus();
      // 2. Toujours récupérer les données du tableau de bord si le backend est en ligne, unifiant la vue.
      if (status && !status.is_offline) {
        fetchSimulationData();
        fetchActivityLog();
      }
      // 3. Récupérer l'état de préparation uniquement si le bot est arrêté mais en ligne
      if (status && !status.is_running && !status.is_offline) {
        fetchReadiness();
      }
      if (isPageLoading) setIsPageLoading(false);
    }

    updateData(); // Appel initial
    const interval = setInterval(updateData, 5000);
    return () => clearInterval(interval);
  }, [fetchBotStatus, isPageLoading, fetchSimulationData, fetchActivityLog, fetchReadiness]); // Correctly include all dependencies

  const handleStart = async (mode) => {
    if (isLoading || botStatus?.is_running || botStatus?.is_offline) return;
    setIsLoading(true);
    setError('');
    try {
      const response = await fetch(`${API_URL}/api/bot/start?mode=${mode}`, { method: 'POST' });
      if (!response.ok) throw new Error((await response.json()).detail || 'Failed to start bot');
      setTimeout(fetchBotStatus, 1000);
    } catch (err) { setError(err.message); } 
    finally { setIsLoading(false); }
  };

  const handleStartRealWithConfirmation = () => {
    const confirmationMessage = "Êtes-vous sûr de vouloir démarrer le bot en MODE RÉEL ?\n\n" +
                                "Cela engagera des fonds réels et peut entraîner des pertes financières. " +
                                "Assurez-vous d'avoir bien testé en mode simulation et de comprendre les risques.";
    
    if (window.confirm(confirmationMessage)) {
      handleStart('real');
    }
  };

  const handleStop = async () => {
    if (isLoading || !botStatus?.is_running || botStatus?.is_offline) return;
    setIsLoading(true);
    setError('');
    try {
      const response = await fetch(`${API_URL}/api/bot/stop`, { method: 'POST' });
      if (!response.ok) throw new Error((await response.json()).detail || 'Échec de l\'arrêt du bot');
      setTimeout(fetchBotStatus, 1000);
    } catch (err) { setError(err.message); } 
    finally { setIsLoading(false); }
  };

  if (isPageLoading) {
    return <div className="flex items-center justify-center h-[calc(100vh-4rem)]"><p className="text-gray-500">Connexion au serveur...</p></div>;
  }

  // S'assurer que trade_history est toujours un tableau pour éviter les erreurs de rendu.
  const tradeHistory = Array.isArray(simulationData?.trade_history) ? simulationData.trade_history : [];

  const chartData = {
    labels: tradeHistory.map(t => new Date(t.timestamp * 1000).toLocaleTimeString()),
    datasets: [
      {
      label: 'Profit/Perte Cumulé (SOL)', // Le backend doit fournir l'historique du PnL
      // Pour l'instant, on affiche une ligne plate si l'historique n'est pas là
      data: tradeHistory.length > 0 ? tradeHistory.map(t => t.cumulative_pnl) : [],
      borderColor: 'rgb(14, 165, 233)', // sky-500
      backgroundColor: 'rgba(14, 165, 233, 0.1)',
      tension: 0.3,
      fill: true,
    }],
  };

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
        {/* Colonne de gauche: Contrôles et Statut */}
        <div className="lg:col-span-1 space-y-8">
          <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Centre de Contrôle</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between bg-gray-100 p-3 rounded-lg" data-testid="bot-status-display">
                <span className="text-sm text-gray-600">Statut du Bot</span>
                <span className={`px-3 py-1 text-xs font-medium rounded-full ${botStatus?.is_running ? 'bg-green-100 text-green-800' : botStatus?.is_offline ? 'bg-yellow-100 text-yellow-800' : 'bg-red-100 text-red-800'}`}>
                  {botStatus?.is_running ? `Actif (${botStatus?.current_mode})` : botStatus?.is_offline ? 'Déconnecté' : 'Arrêté'}
                </span>
              </div>
              <div className="pt-2">
                {botStatus?.is_running ? (
                  <button onClick={handleStop} disabled={isLoading} className="w-full bg-red-600 hover:bg-red-700 text-white font-semibold py-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                    {isLoading ? 'Arrêt en cours...' : 'Arrêter le Bot'}
                  </button>
                ) : (
                  <div className="grid grid-cols-2 gap-3">
                    <button onClick={() => handleStart('simulation')} disabled={isLoading} className="w-full bg-sky-600 hover:bg-sky-700 text-white font-semibold py-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                      {isLoading ? '...' : 'Simulation'}
                    </button>
                    <button onClick={handleStartRealWithConfirmation} disabled={isLoading || !readiness.is_ready} className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-semibold py-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                      {isLoading ? '...' : 'Mode Réel'}
                    </button>
                  </div>
                )}
              </div>
              {error && !botStatus.is_running && <p className="text-xs text-red-600 text-center pt-2">{error}</p>}
            </div>
          </div>

          {!botStatus?.is_offline && (
            <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Performance Globale</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-baseline"><span className="text-sm text-gray-500">Profit / Perte</span><span className={`font-semibold ${simulationData?.profit_loss_sol >= 0 ? 'text-green-600' : 'text-red-600'}`}>{simulationData?.profit_loss_sol?.toFixed(4) || '0.0000'} SOL</span></div>
                <div className="flex justify-between items-baseline"><span className="text-sm text-gray-500">Trades Complétés</span><span className="font-semibold text-gray-800">{simulationData?.total_trades || 0}</span></div>
                <div className="flex justify-between items-baseline"><span className="text-sm text-gray-500">Positions Ouvertes</span><span className="font-semibold text-gray-800">{simulationData?.held_tokens_count || 0}</span></div>
              </div>
            </div>
          )}

        </div>

        {/* Colonne de droite: Graphiques et Logs */}
        <div className="lg:col-span-2 space-y-8">
          {/* Performance en Temps Réel */}
          <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Performance en Temps Réel</h3>
            <div className="h-80">
              {tradeHistory.length > 0 ? (
                <Line data={chartData} options={{ responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false } }, scales: { x: { ticks: { color: '#6b7280' } }, y: { ticks: { color: '#6b7280' } } } }} />
              ) : (
                <div className="flex items-center justify-center h-full text-gray-400 text-sm">
                  {botStatus?.is_running ? 'En attente de données de trading...' : 'Démarrez une session pour voir les performances.'}
                </div>
              )}
            </div>
          </div>

          {/* Statistiques détaillées */}
          <DetailedStats />

          {/* Statut de l'optimiseur IA */}
          <AIOptimizerStatus />

          {/* Logs d'activité */}
          {Array.isArray(activityLog) && activityLog.length > 0 && (
            <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Logs d'Activité Récente</h3>
              <div className="max-h-64 overflow-y-auto space-y-2">
                {activityLog.slice(0, 10).map((log) => (
                  <div key={log.id} className="text-sm p-2 bg-gray-50 rounded">
                    <span className="text-gray-500">{new Date(log.timestamp * 1000).toLocaleString()}</span>
                    <span className="ml-2">{log.message}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Test du bot */}
          <BotTesting />

          {/* Métriques de performance */}
          <PerformanceMetrics />

        </div>
      </div>
    </div>
  );
}

function Dashboard() {
  return (
    <ErrorBoundary>
      <DashboardContent />
    </ErrorBoundary>
  );
}

export default Dashboard;
