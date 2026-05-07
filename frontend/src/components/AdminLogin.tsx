import { useState } from 'react';
import { getAdminPassword, setAdminPassword } from '../api';

export function AdminLogin({ onDone }: { onDone: () => void }) {
  const current = getAdminPassword();
  const [password, setPassword] = useState(current);

  const handleSave = () => {
    setAdminPassword(password.trim());
    onDone();
    window.location.reload();
  };

  const handleLogout = () => {
    setAdminPassword('');
    onDone();
    window.location.reload();
  };

  return (
    <div className="bg-white rounded-2xl shadow-sm p-5">
      <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wider mb-3">
        Admin Access
      </h3>
      <div className="flex gap-3">
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSave()}
          placeholder="Admin password"
          className="flex-1 px-4 py-2 border border-gray-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-gray-900 focus:border-transparent"
        />
        <button
          onClick={handleSave}
          className="px-4 py-2 bg-gray-900 text-white rounded-xl text-sm font-medium hover:bg-gray-800 transition-colors"
        >
          Save
        </button>
        {current && (
          <button
            onClick={handleLogout}
            className="px-4 py-2 border border-gray-200 text-gray-500 rounded-xl text-sm font-medium hover:bg-gray-50 transition-colors"
          >
            Logout
          </button>
        )}
      </div>
    </div>
  );
}
