import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function AIOptimizerDetails() {
  const [details, setDetails] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchDetails = async () => {
      try {
        const response = await fetch(`${API_URL}/api/bot/optimizer/status`);
        if (!response.ok) {
          throw new Error('Failed to fetch optimizer details');
        }
        const data = await response.json();
        setDetails(data);
        setError('');
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch optimizer details:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchDetails();
    const interval = setInterval(fetchDetails, 20000); // Refresh every 20 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Détails de l'Optimiseur IA</h3>
        <div className="text-center text-gray-500">Chargement des détails...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Détails de l'Optimiseur IA</h3>
        <div className="text-center text-red-500">Erreur: {error}</div>
      </div>
    );
  }

  if (!details) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Détails de l'Optimiseur IA</h3>
        <div className="text-center text-gray-500">Aucune donnée disponible</div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Détails de l'Optimiseur IA</h3>
      
      <div className="space-y-6">
        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">État Actuel</h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <p className="text-sm text-gray-500">Mode</p>
              <p className="text-lg font-bold text-blue-600">{details.mode || 'N/A'}</p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">Optimisations</p>
              <p className="text-lg font-bold text-green-600">{details.optimizations_count || 0}</p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">État</p>
              <p className={`text-lg font-bold ${details.is_active ? 'text-green-600' : 'text-yellow-600'}`}>
                {details.is_active ? 'Actif' : 'Inactif'}
              </p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">Prochain cycle</p>
              <p className="text-lg font-semibold">{details.next_cycle || 'N/A'}</p>
            </div>
          </div>
        </div>

        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">Paramètres Actuels</h4>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600">Multiplicateur de vente</span>
              <span className="font-semibold">{details.current_sell_multiplier || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Trailing stop</span>
              <span className="font-semibold">{details.current_trailing_stop || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Montant par trade</span>
              <span className="font-semibold">{details.current_trade_amount || 'N/A'}</span>
            </div>
          </div>
        </div>

        {details.last_optimization && (
          <div>
            <h4 className="text-md font-semibold text-gray-700 mb-3">Dernière Optimisation</h4>
            <div className="text-sm text-gray-600 space-y-2">
              <p>Date: {new Date(details.last_optimization.timestamp * 1000).toLocaleString()}</p>
              <p>P&L moyen: {details.last_optimization.avg_pnl?.toFixed(2)}%</p>
              <p>Changements: {details.last_optimization.changes || 'Aucun'}</p>
              {details.last_optimization.reason && (
                <p>Raison: {details.last_optimization.reason}</p>
              )}
            </div>
          </div>
        )}

        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">Historique des Optimisations</h4>
          <div className="text-sm text-gray-600">
            <p>Total des optimisations: {details.total_optimizations || 0}</p>
            <p>Optimisations réussies: {details.successful_optimizations || 0}</p>
            <p>Optimisations échouées: {details.failed_optimizations || 0}</p>
            <p>Taux de succès: {details.success_rate ? `${details.success_rate.toFixed(1)}%` : 'N/A'}</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default AIOptimizerDetails;