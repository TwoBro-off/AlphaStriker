import React, { useState, useEffect, useCallback } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

/**
 * Composant pour afficher l'activité de trading sous forme de tableau.
 */
function TradingActivity() {
  const [trades, setTrades] = useState([]);
  const [error, setError] = useState('');

  useEffect(() => {
    const fetchTrades = async () => {
      try {
        const response = await fetch(`${API_URL}/api/bot/activity`);
        if (!response.ok) throw new Error("Échec de la récupération de l'activité");
        const data = await response.json();
        setTrades(Array.isArray(data) ? data : []);
        setError('');
      } catch (err) {
        setError(err.message);
      }
    };

    fetchTrades();
    const interval = setInterval(fetchTrades, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="overflow-x-auto">
      {error && <div className="mb-4 p-3 bg-red-100 text-red-700 rounded-lg text-sm">Erreur: {error}</div>}
      {trades.length > 0 ? (
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              {['Token', 'Date Achat', 'Prix Achat', 'Prix Vente', 'P&L (SOL)', 'P&L (%)', 'Mode'].map(header => (
                <th key={header} scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{header}</th>
              ))}
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {trades.map((trade) => (
              <tr key={trade.id}>
                <td className="px-6 py-4 whitespace-nowrap">
                  <a href={`https://solscan.io/token/${trade.token_address}`} target="_blank" rel="noopener noreferrer" className="text-sm font-mono text-sky-600 hover:text-sky-800">
                    {`${trade.token_address.substring(0, 4)}...${trade.token_address.substring(trade.token_address.length - 4)}`}
                  </a>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{new Date(trade.buy_timestamp * 1000).toLocaleString()}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{trade.buy_price.toFixed(8)}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{trade.sell_price != null ? trade.sell_price.toFixed(8) : <span className="text-gray-400">En cours</span>}</td>
                <td className={`px-6 py-4 whitespace-nowrap text-sm font-semibold ${trade.pnl_sol == null ? 'text-gray-600' : trade.pnl_sol >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {trade.pnl_sol != null ? trade.pnl_sol.toFixed(4) : '-'}
                </td>
                <td className={`px-6 py-4 whitespace-nowrap text-sm font-semibold ${trade.pnl_percent == null ? 'text-gray-600' : trade.pnl_percent >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {trade.pnl_percent != null ? `${trade.pnl_percent.toFixed(2)}%` : '-'}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${trade.run_mode === 'REAL' ? 'bg-indigo-100 text-indigo-800' : 'bg-sky-100 text-sky-800'}`}>
                    {trade.run_mode}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <div className="text-center py-12 px-6">
          <p className="text-gray-500">En attente des premières activités de trading...</p>
        </div>
      )}
    </div>
  );
}

/**
 * Page principale d'activité en direct avec des onglets.
 */
function LiveActivityPage() {
  const [activeTab, setActiveTab] = useState('trading');

  const tabs = [{ id: 'trading', label: 'Activité de Trading' }];

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Activité en Direct</h1>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6" aria-label="Tabs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`${
                  activeTab === tab.id
                    ? 'border-sky-500 text-sky-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
              >
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {activeTab === 'trading' && <TradingActivity />}
        </div>
      </div>
    </div>
  );
}

export default LiveActivityPage;