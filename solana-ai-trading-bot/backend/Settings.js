import React, { useState, useEffect } from 'react';
import { toast, ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

function Settings() {
    const [settings, setSettings] = useState({});
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        fetch('/api/env')
            .then(res => res.json())
            .then(data => {
                setSettings(data);
                setIsLoading(false);
            })
            .catch(err => {
                setError('Impossible de charger les paramètres.');
                setIsLoading(false);
            });
    }, []);

    const handleInputChange = (e) => {
        const { name, value } = e.target;
        setSettings(prev => ({ ...prev, [name]: value }));
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        const toastId = toast.loading("Sauvegarde des paramètres...");

        const updates = Object.keys(settings).map(key =>
            fetch(`/api/env/update?key=${encodeURIComponent(key)}&value=${encodeURIComponent(settings[key])}`, {
                method: 'POST',
            })
        );

        try {
            await Promise.all(updates);
            toast.update(toastId, { render: "Paramètres sauvegardés avec succès !", type: "success", isLoading: false, autoClose: 3000 });
        } catch (err) {
            toast.update(toastId, { render: `Erreur lors de la sauvegarde: ${err.message}`, type: "error", isLoading: false, autoClose: 5000 });
        }
    };

    if (isLoading) return <p>Chargement des paramètres...</p>;
    if (error) return <p className="text-red-500">{error}</p>;

    const renderInput = (key, label) => (
        <div key={key} className="mb-4">
            <label htmlFor={key} className="block text-sm font-medium text-gray-700">{label}</label>
            <input
                type="text"
                id={key}
                name={key}
                value={settings[key] || ''}
                onChange={handleInputChange}
                className="mt-1 block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            />
        </div>
    );

    return (
        <div className="p-6 bg-gray-50 min-h-screen">
            <ToastContainer position="top-right" autoClose={5000} hideProgressBar={false} />
            <h1 className="text-3xl font-bold text-gray-800 mb-6">Paramètres du Bot</h1>

            <form onSubmit={handleSubmit} className="bg-white p-8 rounded-lg shadow-md max-w-2xl mx-auto">
                <h2 className="text-xl font-semibold text-gray-700 mb-6">Clés API et Wallets</h2>
                
                {renderInput("OPENROUTER_API_KEY", "Clé API OpenRouter")}
                {renderInput("GEMINI_MODEL", "Modèle IA (ex: google/gemini-pro)")}
                {renderInput("TRUSTWALLET_ADDRESS", "Adresse du Wallet de Sécurité (TrustWallet)")}

                <hr className="my-6" />

                <h2 className="text-xl font-semibold text-gray-700 mb-6">Paramètres de Trading</h2>

                {renderInput("BUY_AMOUNT_SOL", "Montant d'achat par trade (SOL)")}
                {renderInput("SELL_MULTIPLIER", "Multiplicateur de Take Profit (ex: 2.0 pour x2)")}

                <div className="mt-8 flex justify-end">
                    <button
                        type="submit"
                        className="px-6 py-2 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                    >
                        Sauvegarder les modifications
                    </button>
                </div>
            </form>
        </div>
    );
}
import React, { useState, useEffect } from 'react';
import { toast, ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

function Settings() {
    const [settings, setSettings] = useState({});
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        fetch('/api/env')
            .then(res => res.json())
            .then(data => {
                setSettings(data);
                setIsLoading(false);
            })
            .catch(err => {
                setError('Impossible de charger les paramètres.');
                setIsLoading(false);
            });
    }, []);

    const handleInputChange = (e) => {
        const { name, value } = e.target;
        setSettings(prev => ({ ...prev, [name]: value }));
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        const toastId = toast.loading("Sauvegarde des paramètres...");

        const updates = Object.keys(settings).map(key =>
            fetch(`/api/env/update?key=${encodeURIComponent(key)}&value=${encodeURIComponent(settings[key])}`, {
                method: 'POST',
            })
        );

        try {
            await Promise.all(updates);
            toast.update(toastId, { render: "Paramètres sauvegardés avec succès !", type: "success", isLoading: false, autoClose: 3000 });
        } catch (err) {
            toast.update(toastId, { render: `Erreur lors de la sauvegarde: ${err.message}`, type: "error", isLoading: false, autoClose: 5000 });
        }
    };

    if (isLoading) return <p>Chargement des paramètres...</p>;
    if (error) return <p className="text-red-500">{error}</p>;

    const renderInput = (key, label) => (
        <div key={key} className="mb-4">
            <label htmlFor={key} className="block text-sm font-medium text-gray-700">{label}</label>
            <input
                type="text"
                id={key}
                name={key}
                value={settings[key] || ''}
                onChange={handleInputChange}
                className="mt-1 block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            />
        </div>
    );

    return (
        <div className="p-6 bg-gray-50 min-h-screen">
            <ToastContainer position="top-right" autoClose={5000} hideProgressBar={false} />
            <h1 className="text-3xl font-bold text-gray-800 mb-6">Paramètres du Bot</h1>

            <form onSubmit={handleSubmit} className="bg-white p-8 rounded-lg shadow-md max-w-2xl mx-auto">
                <h2 className="text-xl font-semibold text-gray-700 mb-6">Clés API et Wallets</h2>
                
                {renderInput("OPENROUTER_API_KEY", "Clé API OpenRouter")}
                {renderInput("GEMINI_MODEL", "Modèle IA (ex: google/gemini-pro)")}
                {renderInput("TRUSTWALLET_ADDRESS", "Adresse du Wallet de Sécurité (TrustWallet)")}

                <hr className="my-6" />

                <h2 className="text-xl font-semibold text-gray-700 mb-6">Paramètres de Trading</h2>

                {renderInput("BUY_AMOUNT_SOL", "Montant d'achat par trade (SOL)")}
                {renderInput("SELL_MULTIPLIER", "Multiplicateur de Take Profit (ex: 2.0 pour x2)")}

                <div className="mt-8 flex justify-end">
                    <button
                        type="submit"
                        className="px-6 py-2 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                    >
                        Sauvegarder les modifications
                    </button>
                </div>
            </form>
        </div>
    );
}

export default Settings;

export default Settings;