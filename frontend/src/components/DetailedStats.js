import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function DetailedStats() {
  const [stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch(`${API_URL}/api/bot/activity`);
        if (!response.ok) {
          throw new Error('Failed to fetch stats');
        }
        const data = await response.json();
        setStats(data);
        setError('');
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch stats:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statistiques Détaillées</h3>
        <div className="text-center text-gray-500">Chargement des statistiques...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statistiques Détaillées</h3>
        <div className="text-center text-red-500">Erreur: {error}</div>
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statistiques Détaillées</h3>
        <div className="text-center text-gray-500">Aucune donnée disponible</div>
      </div>
    );
  }

  const completedTrades = stats.filter(t => t.pnl_percent != null);
  const totalCompleted = completedTrades.length;

  const winRate = totalCompleted > 0 ?
    (completedTrades.filter(t => t.pnl_percent > 0).length / totalCompleted * 100) : 0;

  const avgPnl = totalCompleted > 0 ?
    (completedTrades.reduce((sum, t) => sum + t.pnl_percent, 0) / totalCompleted) : 0;

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Statistiques Détaillées</h3>
      
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="text-center">
          <p className="text-sm text-gray-500">Win Rate</p>
          <p className="text-2xl font-bold text-blue-600">{winRate.toFixed(1)}%</p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">P&L Moyen</p>
          <p className={`text-2xl font-bold ${avgPnl >= 0 ? 'text-green-600' : 'text-red-600'}`}>
            {avgPnl.toFixed(2)}%
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Meilleur Trade</p>
          <p className="text-2xl font-bold text-green-600">
            {Math.max(...(completedTrades.map(t => t.pnl_percent) || [0])).toFixed(2)}%
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Pire Trade</p>
          <p className="text-2xl font-bold text-red-600">
            {Math.min(...(completedTrades.map(t => t.pnl_percent) || [0])).toFixed(2)}%
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-2">Performance par Mode</h4>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Mode Simulation</span>
              <span className="text-sm font-semibold">
                {stats.filter(t => t.run_mode === 'DEMO').length || 0} trades
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Mode Réel</span>
              <span className="text-sm font-semibold">
                {stats.filter(t => t.run_mode === 'REAL').length || 0} trades
              </span>
            </div>
          </div>
        </div>

        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-2">Répartition des Résultats</h4>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Trades Gagnants</span>
              <span className="text-sm font-semibold text-green-600">
                {completedTrades.filter(t => t.pnl_percent > 0).length || 0}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Trades Perdants</span>
              <span className="text-sm font-semibold text-red-600">
                {completedTrades.filter(t => t.pnl_percent < 0).length || 0}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Trades Neutres</span>
              <span className="text-sm font-semibold text-gray-600">
                {completedTrades.filter(t => t.pnl_percent === 0).length || 0}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default DetailedStats;
