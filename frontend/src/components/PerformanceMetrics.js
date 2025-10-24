import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function PerformanceMetrics() {
  const [metrics, setMetrics] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        const response = await fetch(`${API_URL}/metrics`);
        if (!response.ok) {
          throw new Error('Failed to fetch metrics');
        }
        const data = await response.text();
        
        // Parse Prometheus metrics
        const parsedMetrics = {};
        data.split('\n').forEach(line => {
          if (line && !line.startsWith('#')) {
            const [name, value] = line.split(' ');
            if (name && value) {
              parsedMetrics[name] = parseFloat(value);
            }
          }
        });
        
        setMetrics(parsedMetrics);
        setError('');
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch metrics:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, 10000); // Refresh every 10 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Métriques de Performance</h3>
        <div className="text-center text-gray-500">Chargement des métriques...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Métriques de Performance</h3>
        <div className="text-center text-red-500">Erreur: {error}</div>
      </div>
    );
  }

  if (!metrics || Object.keys(metrics).length === 0) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Métriques de Performance</h3>
        <div className="text-center text-gray-500">Aucune métrique disponible</div>
      </div>
    );
  }

  const formatValue = (value, unit = '') => {
    if (typeof value !== 'number') return 'N/A';
    if (unit === 'SOL') return `${value.toFixed(4)} ${unit}`;
    if (unit === 'ms') return `${Math.round(value)} ${unit}`;
    if (unit === '%') return `${value.toFixed(1)}${unit}`;
    return value.toString();
  };

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Métriques de Performance</h3>
      
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="text-center">
          <p className="text-sm text-gray-500">Trades Totaux</p>
          <p className="text-2xl font-bold text-blue-600">
            {formatValue(metrics.total_trades_completed)}
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">P&L Cumulé</p>
          <p className={`text-2xl font-bold ${(metrics.pnl_cumulated || 0) >= 0 ? 'text-green-600' : 'text-red-600'}`}>
            {formatValue(metrics.pnl_cumulated, 'SOL')}
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Latence Achat</p>
          <p className="text-2xl font-bold text-purple-600">
            {formatValue(metrics.buy_latency_total_ms, 'ms')}
          </p>
        </div>
        <div className="text-center">
          <p className="text-sm text-gray-500">Latence Vente</p>
          <p className="text-2xl font-bold text-orange-600">
            {formatValue(metrics.sell_latency_total_ms, 'ms')}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">Performance Trading</h4>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Durée moyenne des trades</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.trade_duration_total_ms, 'ms')}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Trades par minute</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.trades_per_minute || 0)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Taux de succès</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.success_rate || 0, '%')}
              </span>
            </div>
          </div>
        </div>

        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">Système</h4>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Uptime</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.uptime_seconds || 0, 's')}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Mémoire utilisée</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.memory_usage || 0, 'MB')}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">CPU</span>
              <span className="text-sm font-semibold">
                {formatValue(metrics.cpu_usage || 0, '%')}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default PerformanceMetrics;
