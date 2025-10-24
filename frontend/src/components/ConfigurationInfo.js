import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function ConfigurationInfo() {
  const [config, setConfig] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const response = await fetch(`${API_URL}/api/bot/config`);
        if (!response.ok) {
          throw new Error('Failed to fetch configuration');
        }
        const data = await response.json();
        setConfig(data);
        setError('');
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch configuration:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchConfig();
    const interval = setInterval(fetchConfig, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration</h3>
        <div className="text-center text-gray-500">Chargement de la configuration...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration</h3>
        <div className="text-center text-red-500">Erreur: {error}</div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration</h3>
        <div className="text-center text-gray-500">Aucune configuration disponible</div>
      </div>
    );
  }

  const getStatusColor = (status) => {
    switch (status) {
      case 'OK': return 'text-green-600 bg-green-100';
      case 'ERROR': return 'text-red-600 bg-red-100';
      case 'WARNING': return 'text-yellow-600 bg-yellow-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'OK': return '✓';
      case 'ERROR': return '✗';
      case 'WARNING': return '⚠';
      default: return '?';
    }
  };

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration du Bot</h3>
      
      <div className="space-y-4">
        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">État de Préparation</h4>
          <div className="space-y-2">
            {config.checks && Object.entries(config.checks).map(([key, value]) => (
              <div key={key} className="flex items-center justify-between p-2 rounded-lg">
                <span className="text-sm text-gray-600 capitalize">
                  {key.replace(/_/g, ' ')}
                </span>
                <span className={`px-2 py-1 rounded-full text-xs font-semibold ${getStatusColor(value)}`}>
                  {getStatusIcon(value)} {value}
                </span>
              </div>
            ))}
          </div>
        </div>

        {config.missing_items && config.missing_items.length > 0 && (
          <div>
            <h4 className="text-md font-semibold text-gray-700 mb-3">Éléments Manquants</h4>
            <div className="space-y-2">
              {config.missing_items.map((item, index) => (
                <div key={index} className="flex items-center p-2 bg-yellow-50 rounded-lg">
                  <span className="text-yellow-600 mr-2">⚠</span>
                  <span className="text-sm text-yellow-800">{item}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        <div>
          <h4 className="text-md font-semibold text-gray-700 mb-3">Informations Système</h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600">Mode de fonctionnement</span>
              <span className="font-semibold">{config.mode || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Version</span>
              <span className="font-semibold">{config.version || 'N/A'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Dernière mise à jour</span>
              <span className="font-semibold">
                {config.last_update ? new Date(config.last_update).toLocaleString() : 'N/A'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600">Statut global</span>
              <span className={`font-semibold ${getStatusColor(config.is_ready ? 'OK' : 'ERROR')}`}>
                {config.is_ready ? 'Prêt' : 'Non prêt'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ConfigurationInfo;
