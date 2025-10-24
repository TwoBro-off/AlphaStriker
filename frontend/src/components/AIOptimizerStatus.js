import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function AIOptimizerStatus() {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const response = await fetch(`${API_URL}/api/ai/status`);
        if (!response.ok) {
          throw new Error('Failed to fetch optimizer status');
        }
        const data = await response.json();
        setStatus(data);
        setError('');
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch optimizer status:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 10000); // Refresh every 10 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statut de l'Optimiseur IA</h3>
        <div className="text-center text-gray-500">Chargement...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statut de l'Optimiseur IA</h3>
        <div className="text-center text-red-500">Erreur: {error}</div>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Statut de l'Optimiseur IA</h3>
        <div className="text-center text-gray-500">Aucune donnée disponible</div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Statut de l'Optimiseur IA</h3>
      
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="text-center">
          <p className="text-sm text-gray-500">Mode</p>
          <p className="text-lg font-bold text-blue-600">{status.mode || 'DEMO'}</p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Optimisations</p>
          <p className="text-lg font-bold text-green-600">{status.optimizations_count || 0}</p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">État</p>
          <p className={`text-lg font-bold ${status.is_active ? 'text-green-600' : 'text-yellow-600'}`}>
            {status.is_active ? 'Actif' : 'En attente'}
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Prochain cycle</p>
          <p className="text-lg font-semibold">{status.next_cycle || 'N/A'}</p>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-2">Paramètres Actuels</h4>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600">Multiplicateur de vente</span>
              <span className="font-semibold">{status.current_sell_multiplier || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Trailing stop</span>
              <span className="font-semibold">{status.current_trailing_stop || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Montant par trade</span>
              <span className="font-semibold">{status.current_trade_amount || 'N/A'}</span>
            </div>
          </div>
        </div>

        {status.last_optimization && (
          <div>
            <h4 className="text-md font-semibold text-gray-700 mb-2">Dernière Optimisation</h4>
            <div className="text-sm text-gray-600">
              <p>Date: {new Date(status.last_optimization.timestamp * 1000).toLocaleString()}</p>
              <p>P&L moyen: {status.last_optimization.avg_pnl?.toFixed(2)}%</p>
              <p>Changements: {status.last_optimization.changes || 'Aucun'}</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default AIOptimizerStatus;
