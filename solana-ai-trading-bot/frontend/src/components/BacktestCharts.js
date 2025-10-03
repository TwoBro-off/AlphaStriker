import React from "react";
import { Line, Bar } from "react-chartjs-2";
import { Chart, CategoryScale, LinearScale, PointElement, LineElement, BarElement, Tooltip, Legend } from "chart.js";

Chart.register(CategoryScale, LinearScale, PointElement, LineElement, BarElement, Tooltip, Legend);

export function PnLChart({ results }) {
  if (!results || !Array.isArray(results) || results.length === 0) return null;
  let cumPnL = 0;
  const labels = results.map((r, i) => `Run ${i + 1}`);
  const data = results.map(r => {
    cumPnL += r.performance?.total_profit || 0;
    return cumPnL;
  });
  return (
    <div className="mb-4">
      <h4 className="font-semibold mb-1">Courbe PnL cumulée</h4>
      <Line data={{
        labels,
        datasets: [{
          label: "PnL cumulé",
          data,
          fill: false,
          borderColor: "#2563eb",
          backgroundColor: "#60a5fa",
          tension: 0.2
        }]
      }} options={{ responsive: true, plugins: { legend: { display: false } } }} height={120} />
    </div>
  );
}

export function DrawdownChart({ results }) {
  if (!results || !Array.isArray(results) || results.length === 0) return null;
  let maxPnL = 0, pnl = 0;
  const drawdowns = results.map(r => {
    pnl += r.performance?.total_profit || 0;
    if (pnl > maxPnL) maxPnL = pnl;
    return maxPnL - pnl;
  });
  return (
    <div className="mb-4">
      <h4 className="font-semibold mb-1">Drawdown par run</h4>
      <Bar data={{
        labels: results.map((_, i) => `Run ${i + 1}`),
        datasets: [{
          label: "Drawdown",
          data: drawdowns,
          backgroundColor: "#f87171"
        }]
      }} options={{ responsive: true, plugins: { legend: { display: false } } }} height={120} />
    </div>
  );
}

export function TradesHeatmap({ results }) {
  if (!results || !Array.isArray(results) || results.length === 0) return null;
  // Simple heatmap: nombre de trades par run
  const tradeCounts = results.map(r => r.trades?.length || 0);
  return (
    <div className="mb-4">
      <h4 className="font-semibold mb-1">Heatmap : nombre de trades par run</h4>
      <Bar data={{
        labels: results.map((_, i) => `Run ${i + 1}`),
        datasets: [{
          label: "# Trades",
          data: tradeCounts,
          backgroundColor: "#34d399"
        }]
      }} options={{ responsive: true, plugins: { legend: { display: false } } }} height={120} />
    </div>
  );
}
