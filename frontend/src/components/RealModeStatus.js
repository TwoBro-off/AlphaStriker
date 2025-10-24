import React, { useState, useEffect } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function StatusItem({ label, status, value, error }) {
    const isOk = status === 'ok';
    return (
        <div className={`flex justify-between items-center p-3 rounded-lg ${isOk ? 'bg-green-50' : 'bg-red-50'}`}>
            <div>
                <span className="font-medium text-gray-800">{label}</span>
                {value && <span className="ml-2 text-sm text-gray-600 font-mono">{value}</span>}
                {error && <p className="text-xs text-red-700 mt-1">{error}</p>}
            </div>
            <span className={`text-2xl ${isOk ? 'text-green-500' : 'text-red-500'}`}>
                {isOk ? '✅' : '❌'}
            </span>
        </div>
    );
}

function RealModeStatus() {
    const [status, setStatus] = useState(null);
    const [loading, setLoading] = useState(true);
    const [botIsRunning, setBotIsRunning] = useState(true);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const response = await fetch(`${API_URL}/api/bot/readiness`);
                if (!response.ok) {
                    throw new Error('Failed to fetch pre-flight status');
                }
                const data = await response.json();
                setStatus(data);

                const botStatusResponse = await fetch(`${API_URL}/api/bot/status`);
                const botStatusData = await botStatusResponse.json();
                setBotIsRunning(botStatusData.is_running);
            } catch (err) {
                console.error(err);
            } finally {
                setLoading(false);
            }
        };

        fetchStatus();
        const interval = setInterval(fetchStatus, 10000); // Refresh every 10 seconds
        return () => clearInterval(interval);
    }, []);

    if (loading) {
        return <div className="p-4 text-center">Vérification du mode réel en cours...</div>;
    }

    if (!status || botIsRunning) {
        return null; // Ne rien afficher si le bot est en cours d'exécution ou si les données ne sont pas chargées
    }

    // Utilisation de l'optional chaining pour éviter les erreurs si 'checks' n'est pas défini
    return (
        <div className="p-6 bg-white rounded-lg shadow-md border-l-4 border-yellow-500 mb-6">
            <h2 className="text-xl font-bold text-gray-800 mb-4">Pré-vérification pour le Mode Réel</h2>
            <div className="space-y-3">
                <StatusItem
                    label="Connexion du Wallet de Trading"
                    status={status.checks?.private_key_set ? 'ok' : 'error'}
                    error={!status.checks?.private_key_set ? 'Clé privée invalide ou non configurée dans .env' : null}
                />
                <StatusItem
                    label="Solde du Wallet de Trading"
                    status={status.checks?.initial_balance_ok ? 'ok' : 'error'}
                    value={typeof status.checks?.balance_sol === 'number' ? `${status.checks.balance_sol.toFixed(4)} SOL` : 'N/A'}
                    error={!status.checks?.initial_balance_ok ? 'Solde insuffisant pour trader.' : null}
                />
                <StatusItem label="Connexion au Nœud RPC Solana" status={status.checks?.rpc_connection_ok ? 'ok' : 'error'} />
                <StatusItem label="Clé API Helius" status={status.checks?.helius_key_set ? 'ok' : 'error'} />
            </div>
        </div>
    );
}

export default RealModeStatus;