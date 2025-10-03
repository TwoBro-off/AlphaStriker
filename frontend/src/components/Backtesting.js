
import React, { useState } from "react";
import { PnLChart, DrawdownChart, TradesHeatmap } from "./BacktestCharts";
import GeminiSuggestions from "./GeminiSuggestions";
import HealthStatus from "./HealthStatus";
import AutoUpdate from "./AutoUpdate";
import SecurityAlerts from "./SecurityAlerts";
import _ from "lodash";
import { saveAs } from "file-saver";
import axios from "axios";
import { Tab } from '@headlessui/react';

const Backtesting = () => {
  const [mode, setMode] = useState("grid");
  const [nIter, setNIter] = useState(30);
  const [paramGrid, setParamGrid] = useState(`{"window": [10, 20, 30], "threshold": [0.01, 0.02, 0.05]}`);
  const [strategy, setStrategy] = useState("trend");
  const [results, setResults] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const defaultTokenList = `[
    {"mint": "token1", "is_scam": false},
    {"mint": "token2", "is_scam": false},
    {"mint": "token3", "is_scam": true}
  ]`;

  const [blockchain, setBlockchain] = useState("solana");
  const [simBlockchain, setSimBlockchain] = useState("solana");

  const handleRunGridSearch = async () => {
    setLoading(true);
    setError("");
    setSuccess("");
    try {
      const response = await axios.post("/api/backtesting/grid_search", {
        strategy,
        param_grid: JSON.parse(paramGrid),
        backtest_config: {},
        search_mode: mode,
        n_iter: nIter,
        blockchain,
      });
      setResults(response.data.results);
      setSuccess("Optimisation terminée !");
    } catch (err) {
      setError("Erreur lors de l'optimisation : " + (err.response?.data?.detail || err.message));
    }
    setLoading(false);
  };
  
  // Pour simulation avancée
  const [tabIdx, setTabIdx] = useState(0);
  const [simWallets, setSimWallets] = useState('sim_wallet1,sim_wallet2');
  const [simStrategies, setSimStrategies] = useState('trend');
  const [simTokenList, setSimTokenList] = useState(defaultTokenList);
  const [simMode, setSimMode] = useState('simulation');
  const [simResults, setSimResults] = useState(null);
  const [simLoading, setSimLoading] = useState(false);
  const [simError, setSimError] = useState("");

  // Export helpers
  const exportResults = (format) => {
    if (!results || !Array.isArray(results)) return;
    let blob, content;
    if (format === "csv") {
      const header = [
        ...Object.keys(results[0].params),
        ...Object.keys(results[0].performance)
      ];
      const rows = results.map(r => [
        ...Object.values(r.params),
        ...Object.values(r.performance)
      ]);
      content = [header.join(",")].concat(rows.map(row => row.join(","))).join("\n");
      blob = new Blob([content], { type: "text/csv;charset=utf-8;" });
    } else if (format === "json") {
      content = JSON.stringify(results, null, 2);
      blob = new Blob([content], { type: "application/json" });
    } else if (format === "html") {
      const header = [
        ...Object.keys(results[0].params),
        ...Object.keys(results[0].performance)
      ];
      const rows = results.map(r => [
        ...Object.values(r.params),
        ...Object.values(r.performance)
      ]);
      let html = `<table border='1'><tr>${header.map(h=>`<th>${h}</th>`).join("")}</tr>`;
      html += rows.map(row => `<tr>${row.map(cell => `<td>${cell}</td>`).join("")}</tr>`).join("");
      html += "</table>";
      blob = new Blob([html], { type: "text/html" });
    }
    saveAs(blob, `backtest_results.${format}`);
  };

  // Analytics helpers avancés
  const getPnL = () => {
    if (!results || !Array.isArray(results)) return 0;
    return results.reduce((acc, r) => acc + (r.performance?.total_profit || 0), 0);
  };
  const getAvgLatency = () => {
    if (!results || !Array.isArray(results)) return 0;
    const lats = results.map(r => r.performance?.latency).filter(Boolean);
    if (!lats.length) return 0;
    return (lats.reduce((a, b) => a + b, 0) / lats.length).toFixed(2);
  };
  const getDrawdown = () => {
    if (!results || !Array.isArray(results)) return 0;
    let maxPnL = 0, maxDrawdown = 0, pnl = 0;
    results.forEach(r => {
      pnl += r.performance?.total_profit || 0;
      if (pnl > maxPnL) maxPnL = pnl;
      const dd = maxPnL - pnl;
      if (dd > maxDrawdown) maxDrawdown = dd;
    });
    return maxDrawdown;
  };
  const getWinrate = () => {
    if (!results || !Array.isArray(results)) return 0;
    let wins = 0, total = 0;
    results.forEach(r => {
      if (r.performance?.total_profit > 0) wins++;
      total++;
    });
    return total ? ((wins / total) * 100).toFixed(1) : 0;
  };
  const getSharpe = () => {
    if (!results || !Array.isArray(results)) return 0;
    const profits = results.map(r => r.performance?.total_profit || 0);
    const mean = _.mean(profits);
    const std = _.std(profits);
    return std ? (mean / std).toFixed(2) : 0;
  };
  const getLeaderboard = () => {
    if (!results || !Array.isArray(results)) return [];
    // Classement par stratégie ou wallet si dispo
    const grouped = _.groupBy(results, r => r.strategy || r.params?.strategy || "unknown");
    return Object.entries(grouped).map(([k, arr]) => ({
      name: k,
      pnl: _.sumBy(arr, r => r.performance?.total_profit || 0),
      count: arr.length
    })).sort((a, b) => b.pnl - a.pnl).slice(0, 5);
  };

  return (
    <div className="p-6 bg-white rounded shadow">
      <HealthStatus />
      <AutoUpdate />
      <h2 className="text-2xl font-bold mb-4">Backtesting & Optimisation Multi-paramètres</h2>
      <div className="mb-4 p-4 bg-blue-50 border-l-4 border-blue-400 text-blue-900 rounded">
        <strong>Optimisez votre stratégie automatiquement !</strong><br />
        Choisissez le mode d’optimisation, le nombre d’itérations (pour random/bayesian), et le param_grid (JSON).<br />
        <span className="text-xs">(Le mode bayesian nécessite <code>scikit-optimize</code> côté serveur)</span>
      </div>
      <div className="mb-4">
        <label className="block font-medium mb-1">Blockchain :</label>
        <select className="border rounded p-2 mr-2" value={blockchain} onChange={e => setBlockchain(e.target.value)}>
          <option value="solana">Solana</option>
          <option value="evm">EVM (Ethereum, Polygon, etc)</option>
          <option value="bsc">BSC (Binance Smart Chain)</option>
        </select>
      </div>
      <div className="mb-4">
        <label className="block font-medium mb-1">Stratégie :</label>
        <select className="border rounded p-2" value={strategy} onChange={e => setStrategy(e.target.value)}>
          <option value="trend">Trend Following</option>
          {/* Ajouter d'autres stratégies ici si besoin */}
        </select>
      </div>
      <div className="mb-4">
        <label className="block font-medium mb-1">Mode d’optimisation :</label>
        <select className="border rounded p-2" value={mode} onChange={e => setMode(e.target.value)}>
          <option value="grid">Grid Search</option>
          <option value="random">Random Search</option>
          <option value="bayesian">Bayesian (si dispo)</option>
        </select>
      </div>
      {(mode === "random" || mode === "bayesian") && (
        <div className="mb-4">
          <label className="block font-medium mb-1">Nombre d’itérations :</label>
          <input type="number" className="border rounded p-2 w-32" value={nIter} min={5} max={200} onChange={e => setNIter(Number(e.target.value))} />
        </div>
      )}
      <div className="mb-4">
        <label className="block font-medium mb-1">Paramètres à optimiser (JSON) :</label>
        <textarea
          className="w-full h-24 p-2 border rounded"
          value={paramGrid}
          onChange={e => setParamGrid(e.target.value)}
          placeholder='{"window": [10, 20, 30], "threshold": [0.01, 0.02, 0.05]}'
        />
      </div>
      <button
        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
        onClick={handleRunGridSearch}
        disabled={loading}
      >
        {loading ? "Optimisation..." : "Lancer l’optimisation"}
      </button>
      {error && <div className="text-red-500 mt-2">{error}</div>}
      {success && <div className="text-green-600 mt-2">{success}</div>}
      {results && Array.isArray(results) && results.length > 0 && (
        <div className="mt-6 overflow-x-auto">
          <SecurityAlerts results={results} />
          <GeminiSuggestions results={results} />
          <div className="flex flex-wrap gap-4 mb-2">
            <button className="bg-green-600 text-white px-3 py-1 rounded" onClick={() => exportResults("csv")}>Export CSV</button>
            <button className="bg-yellow-600 text-white px-3 py-1 rounded" onClick={() => exportResults("json")}>Export JSON</button>
            <button className="bg-pink-600 text-white px-3 py-1 rounded" onClick={() => exportResults("html")}>Export HTML</button>
            <div className="ml-4 text-sm font-semibold">PnL total : <span className="text-blue-700">{getPnL().toFixed(2)}</span></div>
            <div className="ml-4 text-sm font-semibold">Latence moyenne : <span className="text-blue-700">{getAvgLatency()} ms</span></div>
            <div className="ml-4 text-sm font-semibold">Drawdown max : <span className="text-red-700">{getDrawdown().toFixed(2)}</span></div>
            <div className="ml-4 text-sm font-semibold">Winrate : <span className="text-green-700">{getWinrate()}%</span></div>
            <div className="ml-4 text-sm font-semibold">Sharpe : <span className="text-purple-700">{getSharpe()}</span></div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
            <PnLChart results={results} />
            <DrawdownChart results={results} />
            <TradesHeatmap results={results} />
          </div>
          <div className="mb-2">
            <h4 className="font-semibold">Leaderboard (top stratégies)</h4>
            <table className="text-xs border">
              <thead><tr><th className="border px-2">Stratégie</th><th className="border px-2">PnL</th><th className="border px-2">#Runs</th></tr></thead>
              <tbody>
                {getLeaderboard().map((row, i) => (
                  <tr key={i} className={i === 0 ? "bg-yellow-100" : ""}>
                    <td className="border px-2">{row.name}</td>
                    <td className="border px-2">{row.pnl.toFixed(2)}</td>
                    <td className="border px-2">{row.count}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <h3 className="font-semibold mb-2">Top résultats</h3>
          <table className="min-w-full text-xs border">
            <thead>
              <tr>
                {Object.keys(results[0].params).map((k) => <th key={k} className="border px-2 py-1 bg-gray-100">{k}</th>)}
                {Object.keys(results[0].performance).map((k) => <th key={k} className="border px-2 py-1 bg-gray-100">{k}</th>)}
              </tr>
            </thead>
            <tbody>
              {results.slice(0, 20).map((r, i) => (
                <tr key={i} className={i === 0 ? "bg-green-50" : ""}>
                  {Object.values(r.params).map((v, j) => <td key={j} className="border px-2 py-1">{String(v)}</td>)}
                  {Object.values(r.performance).map((v, j) => <td key={j} className="border px-2 py-1">{String(v)}</td>)}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="text-xs text-gray-500 mt-2">Seuls les 20 meilleurs résultats sont affichés.</div>
        </div>
      )}
      <Tab.Group selectedIndex={tabIdx} onChange={setTabIdx}>
        <Tab.List className="flex space-x-2 mb-6">
          <Tab className={({ selected }) => selected ? "bg-blue-600 text-white px-4 py-2 rounded" : "bg-gray-200 text-gray-700 px-4 py-2 rounded"}>Optimisation Paramètres</Tab>
          <Tab className={({ selected }) => selected ? "bg-blue-600 text-white px-4 py-2 rounded" : "bg-gray-200 text-gray-700 px-4 py-2 rounded"}>Simulation avancée (multi-wallet/stratégie)</Tab>
        </Tab.List>
        <Tab.Panels>
          <Tab.Panel>
            {/* ...onglet existant d'optimisation... */}
          </Tab.Panel>
          <Tab.Panel>
            <h2 className="text-2xl font-bold mb-4">Simulation avancée multi-wallet / multi-stratégie</h2>
            <div className="mb-4 p-4 bg-blue-50 border-l-4 border-blue-400 text-blue-900 rounded">
              <strong>Lancez des campagnes de simulation en parallèle sur plusieurs wallets et stratégies.</strong><br />
              Tous les tokens non scam sont achetés en simulation.<br />
              <span className="text-xs">(Pour la simulation uniquement, aucun achat réel n’est effectué.)</span>
            </div>
            <div className="mb-4">
              <label className="block font-medium mb-1">Wallets (séparés par virgule) :</label>
              <input className="border rounded p-2 w-full" value={simWallets} onChange={e => setSimWallets(e.target.value)} placeholder="sim_wallet1,sim_wallet2" />
            </div>
            <div className="mb-4">
              <label className="block font-medium mb-1">Stratégies (séparées par virgule) :</label>
              <input className="border rounded p-2 w-full" value={simStrategies} onChange={e => setSimStrategies(e.target.value)} placeholder="trend,autre" />
            </div>
            <div className="mb-4">
              <label className="block font-medium mb-1">Liste des tokens (JSON) :</label>
              <textarea className="w-full h-24 p-2 border rounded" value={simTokenList} onChange={e => setSimTokenList(e.target.value)} />
            </div>
            <div className="mb-4">
              <label className="block font-medium mb-1">Blockchain :</label>
              <select className="border rounded p-2 mr-2" value={simBlockchain} onChange={e => setSimBlockchain(e.target.value)}>
                <option value="solana">Solana</option>
                <option value="evm">EVM (Ethereum, Polygon, etc)</option>
                <option value="bsc">BSC (Binance Smart Chain)</option>
              </select>
            </div>
            <div className="mb-4">
              <label className="block font-medium mb-1">Mode :</label>
              <select className="border rounded p-2" value={simMode} onChange={e => setSimMode(e.target.value)}>
                <option value="simulation">Simulation</option>
                <option value="real">Réel (bientôt)</option>
              </select>
            </div>
            <button className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700" onClick={async () => {
              setSimLoading(true);
              setSimError("");
              try {
                const response = await axios.post("/api/simulation/multi_strategy", {
                  wallets: simWallets.split(",").map(w => w.trim()),
                  strategies: simStrategies.split(",").map(s => s.trim()),
                  token_list: JSON.parse(simTokenList),
                  backtest_config: { mode: simMode },
                  blockchain: simBlockchain,
                });
                setSimResults(response.data.results);
              } catch (err) {
                setSimError("Erreur simulation : " + (err.response?.data?.detail || err.message));
              }
              setSimLoading(false);
            }} disabled={simLoading}>
              {simLoading ? "Simulation..." : "Lancer la simulation avancée"}
            </button>
            {simError && <div className="text-red-500 mt-2">{simError}</div>}
            {simResults && (
              <div className="mt-6 overflow-x-auto">
                <h3 className="font-semibold mb-2">Résultats par wallet/stratégie</h3>
                <table className="min-w-full text-xs border">
                  <thead>
                    <tr>
                      <th className="border px-2 py-1 bg-gray-100">Wallet</th>
                      <th className="border px-2 py-1 bg-gray-100">Stratégie</th>
                      <th className="border px-2 py-1 bg-gray-100">Profit</th>
                      <th className="border px-2 py-1 bg-gray-100">Trades</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.values(simResults).map((r, i) => (
                      <tr key={i}>
                        <td className="border px-2 py-1">{r.wallet}</td>
                        <td className="border px-2 py-1">{r.strategy}</td>
                        <td className="border px-2 py-1">{r.performance?.total_profit}</td>
                        <td className="border px-2 py-1">{r.trades?.length}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </Tab.Panel>
        </Tab.Panels>
      </Tab.Group>
    </div>
  );
};

export default Backtesting;
