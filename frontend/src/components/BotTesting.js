import React, { useState } from 'react';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

function BotTesting() {
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [tokenMint, setTokenMint] = useState('');

  const handleTestBot = async () => {
    setIsLoading(true);
    setMessage('');
    
    let url = `${API_URL}/api/test`;
    if (tokenMint) {
      url += `?token_mint=${tokenMint}`;
    }

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.detail || 'Failed to start test');
      }
      
      const data = await response.json();
      setMessage(`Test démarré avec succès: ${data.status}`);
    } catch (err) {
      setMessage(`Erreur: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Test du Bot</h3>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-600">Adresse du Token (Optionnel)</label>
          <input
            type="text"
            value={tokenMint}
            onChange={(e) => setTokenMint(e.target.value)}
            placeholder="Ex: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
            className="mt-1 block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-sky-500 focus:border-sky-500 sm:text-sm"
          />
        </div>
        <p className="text-sm text-gray-600">
          Cliquez pour lancer un achat simulé pour le token spécifié (si fourni), ou un test général sinon.
        </p>
        
        <button
          onClick={handleTestBot}
          disabled={isLoading}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? 'Test en cours...' : (tokenMint ? 'Lancer Achat Simulé' : 'Lancer Test Général')}
        </button>
        
        {message && (
          <div className={`p-3 rounded-lg text-sm ${
            message.includes('Erreur') 
              ? 'bg-red-100 text-red-700 border border-red-200' 
              : 'bg-green-100 text-green-700 border border-green-200'
          }`}>
            {message}
          </div>
        )}
        
        <div className="text-xs text-gray-500">
          <p><strong>Note:</strong> Le test simulé utilise les prix du marché en temps réel via Jupiter, mais n'exécute aucune transaction sur la blockchain.</p>
        </div>
      </div>
    </div>
  );
}

export default BotTesting;
