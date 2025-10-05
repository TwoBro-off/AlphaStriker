

import React, { useEffect, useState } from "react";
import PropTypes from "prop-types";

function Settings() {
  const [envVars, setEnvVars] = useState({});
  const [editVars, setEditVars] = useState({});
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState("");

  useEffect(() => {
    fetch("/api/env")
      .then((res) => {
        if (!res.ok) throw new Error("Erreur lors du chargement des paramètres");
        return res.json();
      })
      .then((data) => {
        setEnvVars(data || {});
        setEditVars(data || {});
        setLoading(false);
      })
      .catch(() => {
        setEnvVars({});
        setEditVars({});
        setLoading(false);
        setMessage("Erreur lors du chargement des paramètres.");
      });
  }, []);

  const handleChange = (key, value) => {
    if (Object.prototype.hasOwnProperty.call(editVars, key)) {
      setEditVars({ ...editVars, [key]: value });
    }
  };

  const handleSave = async (key) => {
    setMessage("");
    try {
      const res = await fetch("/api/env/update", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ key, value: editVars[key] })
      });
      if (res.ok) {
        setMessage(`Clé ${key} mise à jour !`);
        setEnvVars({ ...envVars, [key]: editVars[key] });
      } else {
        setMessage("Erreur lors de la mise à jour.");
      }
    } catch (e) {
      setMessage("Erreur lors de la mise à jour.");
    }
  };

  if (loading) return <div className="p-8">Chargement des paramètres...</div>;

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">Paramètres du bot (.env)</h2>
      {message && <div className="mb-4 text-green-500">{message}</div>}
      {Object.keys(envVars).length === 0 ? (
        <div className="text-red-500">Aucune variable d'environnement trouvée.</div>
      ) : (
        <table className="min-w-full bg-gray-800 rounded-lg mb-8">
          <thead>
            <tr>
              <th className="py-2 px-4">Clé</th>
              <th className="py-2 px-4">Valeur</th>
              <th className="py-2 px-4">Action</th>
            </tr>
          </thead>
          <tbody>
            {Object.keys(envVars).map((key) => (
              <tr key={key}>
                <td className="py-2 px-4 font-semibold">{key}</td>
                <td className="py-2 px-4">
                  <input
                    type="text"
                    value={editVars[key] || ""}
                    onChange={(e) => handleChange(key, e.target.value)}
                    className="bg-gray-700 text-white px-2 py-1 rounded w-full"
                  />
                </td>
                <td className="py-2 px-4">
                  <button
                    onClick={() => handleSave(key)}
                    className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-1 rounded"
                  >
                    Enregistrer
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <div className="text-sm text-gray-400">Modifie tes clés API, token GitHub, etc. Les changements sont appliqués au backend.</div>
    </div>
  );

Settings.propTypes = {};
export default Settings;

