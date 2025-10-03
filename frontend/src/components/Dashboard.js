


// Utilisation de fetch pour récupérer les données du backend

function Dashboard() {
  const [dashboardData, setDashboardData] = useState(null);


  useEffect(() => {
    async function fetchDashboardData() {
      try {
        const response = await fetch('http://localhost:8000/api/dashboard');
        if (!response.ok) throw new Error('Erreur API');
        const data = await response.json();
        setDashboardData(data);
      } catch (error) {
        console.error('Erreur lors du chargement du dashboard:', error);
      }
    }
    fetchDashboardData();
  }, []);

  if (!dashboardData) return <div className="text-center p-8">Chargement...</div>;

  return (
    <div className="p-8 bg-gray-900 min-h-screen text-white">
      <h1 className="text-3xl font-bold mb-8">AlphaStriker Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">Latence RPC</h2>
          <p>{dashboardData.rpc_latency} ms</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">Tokens analysés</h2>
          <p>{dashboardData.tokens_scanned}</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">Trades exécutés</h2>
          <p>{dashboardData.trades_executed}</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">Statut analyse IA</h2>
          <p>{dashboardData.ai_analysis_status}</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">Santé système</h2>
          <p>{dashboardData.system_health}</p>
        </div>
      </div>

      <div className="mb-8">
        <h2 className="text-xl font-bold mb-4">Tokens détenus</h2>
        <ul className="list-disc pl-6">
          {dashboardData.tokens_held.map(token => (
            <li key={token.name}>{token.name} : {token.amount}</li>
          ))}
        </ul>
      </div>

      <div>
        <h2 className="text-xl font-bold mb-4">Historique des trades</h2>
        <table className="min-w-full bg-gray-800 rounded-lg">
          <thead>
            <tr>
              <th className="py-2 px-4">ID</th>
              <th className="py-2 px-4">Token</th>
              <th className="py-2 px-4">Type</th>
              <th className="py-2 px-4">Montant</th>
              <th className="py-2 px-4">Date</th>
            </tr>
          </thead>
          <tbody>
            {dashboardData.trade_history.map(trade => (
              <tr key={trade.id} className="border-t border-gray-700">
                <td className="py-2 px-4">{trade.id}</td>
                <td className="py-2 px-4">{trade.token}</td>
                <td className="py-2 px-4">{trade.type}</td>
                <td className="py-2 px-4">{trade.amount}</td>
                <td className="py-2 px-4">{trade.date}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default Dashboard;

