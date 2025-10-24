import React from 'react';
import StrategySettings from './components/StrategySettings.js';

function SettingsPage() {
  return (
    <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
      <div className="space-y-8">
        <h1 className="text-2xl font-bold text-gray-800">Paramètres de la Stratégie</h1>
        <StrategySettings />
      </div>
    </div>
  );
}

export default SettingsPage;