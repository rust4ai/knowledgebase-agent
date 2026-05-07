import { useEffect, useState } from 'react';
import { DocumentList } from './components/DocumentList';
import { Upload } from './components/Upload';
import { QueryPanel } from './components/QueryPanel';
import { AdminLogin } from './components/AdminLogin';
import { fetchConfig, getAdminPassword } from './api';

type Tab = 'query' | 'documents';

export default function App() {
  const [tab, setTab] = useState<Tab>('query');
  const [refreshKey, setRefreshKey] = useState(0);
  const [kbName, setKbName] = useState('Knowledgebase');
  const [showAdmin, setShowAdmin] = useState(false);

  const isAdmin = !!getAdminPassword();

  useEffect(() => {
    fetchConfig().then((c) => setKbName(c.knowledgebase_name));
  }, []);

  return (
    <div className="min-h-screen bg-[#f0f0f3] text-gray-900">
      {/* Header */}
      <header className="bg-white shadow-sm sticky top-0 z-10">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold text-gray-900 tracking-tight">
            {kbName}
          </h1>
          <div className="flex items-center gap-1">
            <nav className="flex gap-1">
              <TabButton active={tab === 'query'} onClick={() => setTab('query')}>
                Query
              </TabButton>
              <TabButton active={tab === 'documents'} onClick={() => setTab('documents')}>
                Documents
              </TabButton>
            </nav>
            <button
              onClick={() => setShowAdmin(!showAdmin)}
              className={`ml-3 px-3 py-2 rounded-xl text-sm transition-colors ${
                isAdmin
                  ? 'text-green-600 hover:bg-green-50'
                  : 'text-gray-300 hover:text-gray-500 hover:bg-gray-100'
              }`}
              title={isAdmin ? 'Admin authenticated' : 'Admin login'}
            >
              {isAdmin ? 'Admin' : 'Login'}
            </button>
          </div>
        </div>
      </header>

      {/* Admin login panel */}
      {showAdmin && (
        <div className="max-w-5xl mx-auto px-6 pt-4">
          <AdminLogin onDone={() => setShowAdmin(false)} />
        </div>
      )}

      {/* Content */}
      <main className="max-w-5xl mx-auto px-6 py-8">
        {tab === 'documents' && (
          <div className="space-y-6">
            {isAdmin && <Upload onUploaded={() => setRefreshKey((k) => k + 1)} />}
            <DocumentList key={refreshKey} isAdmin={isAdmin} />
          </div>
        )}
        {tab === 'query' && <QueryPanel />}
      </main>
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-4 py-2 rounded-xl text-sm font-medium transition-colors ${
        active
          ? 'bg-gray-900 text-white'
          : 'text-gray-400 hover:text-gray-900 hover:bg-gray-100'
      }`}
    >
      {children}
    </button>
  );
}
