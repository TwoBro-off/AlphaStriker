import React, { useState, useEffect } from 'react';
import { SimulationPnLChart } from './BacktestCharts';
import AIStatusPanel from './AIStatusPanel';
import RealModeStatus from './RealModeStatus';

function SimulationDashboard() {
    const [data, setData] = useState(null);
    const [error, setError] = useState(null);
    const [isSimulation, setIsSimulation] = useState(true);
 
    useEffect(() => {
        const fetchData = async () => {
            try {
                const response = await fetch('/api/simulation/dashboard');
                if (!response.ok) {
                    const errorData = await response.json();
                    throw new Error(errorData.detail || 'Failed to fetch simulation data');
                }
                const result = await response.json();
                setData(result);
                setIsSimulation(result.simulation_mode !== false); // API returns true or undefined for sim
                setError(null);
            } catch (err) {
                setError(err.message);
                setData(null);
            }
        };

        fetchData(); // Fetch initial data
        const interval = setInterval(fetchData, 2000); // Refresh every 2 seconds

        return () => clearInterval(interval); // Cleanup on unmount
    }, []);

    if (error) {
        return <div className="p-4 text-center text-red-500 bg-red-100 rounded-lg">Erreur : {error}</div>;
    }

    if (!data) {
        return <div className="p-4 text-center">Chargement des données de simulation...</div>;
    }

    return (
        <div className="p-6 bg-gray-50 min-h-screen">
            <h1 className="text-3xl font-bold text-gray-800 mb-6">
                {isSimulation ? 'Dashboard de Simulation' : 'Dashboard de Trading Réel'}
            </h1>

            {/* Affiche le panneau de vérification uniquement en mode réel */}
            {!isSimulation && <RealModeStatus />}
            
            {/* AI Optimizer Status Panel */}
            <div className="mb-6"><AIStatusPanel /></div>

            {/* Key Metrics */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
                <div className="bg-white p-5 rounded-lg shadow">
                    <h3 className="text-sm font-medium text-gray-500">Profit / Perte (SOL)</h3>
                    <p className={`text-2xl font-semibold ${data.profit_loss_sol >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                        {data.profit_loss_sol.toFixed(4)}
                    </p>
                </div>
                <div className="bg-white p-5 rounded-lg shadow">
                    <h3 className="text-sm font-medium text-gray-500">Trades Total</h3>
                    <p className="text-2xl font-semibold text-gray-800">{data.total_trades}</p>
                </div>
                <div className="bg-white p-5 rounded-lg shadow">
                    <h3 className="text-sm font-medium text-gray-500">Tokens Détenus</h3>
                    <p className="text-2xl font-semibold text-gray-800">{data.held_tokens_count}</p>
                </div>
            </div>

            {/* PnL Chart */}
            <div className="bg-white p-5 rounded-lg shadow mb-6">
                <h2 className="text-xl font-semibold text-gray-700 mb-4">Performance de la Simulation (PnL)</h2>
                <SimulationPnLChart tradeHistory={data.trade_history} />
            </div>

            {/* Held Tokens Table */}
            <div className="bg-white p-5 rounded-lg shadow">
                <h2 className="text-xl font-semibold text-gray-700 mb-4">Positions Ouvertes</h2>
                <div className="overflow-x-auto">
                    <table className="min-w-full divide-y divide-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Token</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Prix d'Achat (SOL)</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Montant Achat (SOL)</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Timestamp Achat</th>
                            </tr>
                        </thead>
                        <tbody className="bg-white divide-y divide-gray-200">
                            {Object.entries(data.held_tokens_details).map(([token, details]) => (
                                <tr key={token}>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-800" title={token}>
                                        {`${token.substring(0, 10)}...`}
                                    </td>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600">{details.buy_price.toFixed(6)}</td>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600">{details.buy_amount.toFixed(4)}</td>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                                        {new Date(details.buy_timestamp * 1000).toLocaleString()}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {Object.keys(data.held_tokens_details).length === 0 && (
                        <p className="text-center py-4 text-gray-500">Aucune position ouverte.</p>
                    )}
                </div>
            </div>
        </div>
    );
}

export default SimulationDashboard;