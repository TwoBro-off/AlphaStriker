import React, { useState, useEffect, useCallback } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function StrategySettings() {
  const [settings, setSettings] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [saveStatus, setSaveStatus] = useState(''); // 'saving', 'saved', 'error'

  const fetchSettings = useCallback(async () => {
    setIsLoading(true);
    try {
      // Cette route lit les paramètres de stratégie actuels.
      const response = await fetch(`${API_URL}/api/strategy/settings`);
      if (!response.ok) throw new Error('Impossible de charger les paramètres de la stratégie');
      const data = await response.json();
      setSettings(data);
      setError('');
    } catch (err) {
      setError(err.message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  const handleInputChange = (key, value) => {
    // Convertir en nombre si le champ est de type number
    const originalValue = settings[key];
    const parsedValue = typeof originalValue === 'number' ? (value === '' ? '' : parseFloat(value)) : value;
    setSettings(prev => ({ ...prev, [key]: parsedValue }));
  };

  const handleSave = async () => {
    setSaveStatus('saving');
    setError('');

    // Validation côté client
    if (settings.buy_amount_sol <= 0) {
      setError("Le montant d'achat doit être positif.");
      setSaveStatus('error');
      return;
    }
    if (settings.sell_multiplier <= 1) {
      setError("Le multiplicateur de vente doit être supérieur à 1.");
      setSaveStatus('error');
      return;
    }

    setError('');
    try {
      const response = await fetch(`${API_URL}/api/strategy/settings`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings),
      });
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.detail || 'Échec de la sauvegarde des paramètres');
      }
      setSaveStatus('saved');
    } catch (err) {
      setSaveStatus('error');
      setError(err.message);
    } finally {
      setTimeout(() => setSaveStatus(''), 2000);
    }
  };

  const renderInput = (key, label, type = 'number', step = "0.01") => (
    <div key={key}>
      <label className="block text-sm font-medium text-gray-600">{label}</label>
      <input
        type={type}
        step={step}
        value={settings?.[key] ?? ''}
        onChange={(e) => handleInputChange(key, e.target.value)}
        className="mt-1 block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-sky-500 focus:border-sky-500 sm:text-sm disabled:bg-gray-50"
        disabled={isLoading || saveStatus === 'saving'}
      />
    </div>
  );

  if (isLoading) return <p className="text-gray-500">Chargement...</p>;
  if (error && !settings) return <p className="text-red-500">Erreur: {error}</p>;

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <p className="text-sm text-gray-500 mb-4">
        Ces paramètres sont chargés depuis votre fichier <code>.env</code> au démarrage, mais peuvent être modifiés ici en temps réel. Les modifications seront perdues au redémarrage du bot.
      </p>
      <div className="space-y-4">
        {renderInput('buy_amount_sol', 'Montant d\'achat (SOL)')}
        {renderInput('sell_multiplier', 'Multiplicateur de vente (ex: 2.0 pour x2)')}
        {renderInput('trailing_stop_percent', 'Trailing Stop (ex: 0.15 pour 15%)', "0.01")}
        <button onClick={handleSave} disabled={isLoading || saveStatus === 'saving'} className="w-full bg-sky-600 hover:bg-sky-700 text-white font-semibold py-2 rounded-lg transition-colors disabled:opacity-50">
          {saveStatus === 'saving' ? 'Sauvegarde...' : saveStatus === 'saved' ? 'Sauvegardé ✓' : 'Sauvegarder les Paramètres'}
        </button>
        {error && <p className="text-red-500 text-sm text-center">{error}</p>}
      </div>
    </div>
  );
}

export default StrategySettings;