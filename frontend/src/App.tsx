import { useState } from 'react';
import { DocumentList } from './components/DocumentList';
import { Upload } from './components/Upload';
import { QueryPanel } from './components/QueryPanel';

type Tab = 'documents' | 'query';

export default function App() {
  const [tab, setTab] = useState<Tab>('documents');
  const [refreshKey, setRefreshKey] = useState(0);

  return (
    <div className="min-h-screen bg-[#f0f0f3] text-gray-900">
      {/* Header */}
      <header className="bg-white shadow-sm sticky top-0 z-10">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold text-gray-900 tracking-tight">
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
          <div className="space-y-6">
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
