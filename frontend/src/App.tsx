import { useState } from 'react';
import { DocumentList } from './components/DocumentList';
import { Upload } from './components/Upload';
import { QueryPanel } from './components/QueryPanel';

type Tab = 'documents' | 'query';

export default function App() {
  const [tab, setTab] = useState<Tab>('documents');
  const [refreshKey, setRefreshKey] = useState(0);

  return (
    <div className="min-h-screen bg-slate-900 text-slate-200">
      {/* Header */}
      <header className="border-b border-slate-700 bg-slate-800/50 backdrop-blur sticky top-0 z-10">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-white tracking-tight">
            Knowledgebase Agent
          </h1>
          <nav className="flex gap-1">
            <TabButton active={tab === 'documents'} onClick={() => setTab('documents')}>
              Documents
            </TabButton>
            <TabButton active={tab === 'query'} onClick={() => setTab('query')}>
              Query
            </TabButton>
          </nav>
        </div>
      </header>

      {/* Content */}
      <main className="max-w-5xl mx-auto px-6 py-8">
        {tab === 'documents' && (
          <div className="space-y-8">
            <Upload onUploaded={() => setRefreshKey((k) => k + 1)} />
            <DocumentList key={refreshKey} />
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
      className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
        active
          ? 'bg-blue-600 text-white'
          : 'text-slate-400 hover:text-white hover:bg-slate-700'
      }`}
    >
      {children}
    </button>
  );
}
