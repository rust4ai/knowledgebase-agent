import { useEffect, useState } from 'react';
import { type Document, listDocuments, deleteDocument } from '../api';

const STATUS_COLORS: Record<string, string> = {
  uploaded: 'bg-amber-100 text-amber-700',
  processing: 'bg-blue-100 text-blue-700',
  indexed: 'bg-green-100 text-green-700',
  failed: 'bg-red-100 text-red-700',
};

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function timeAgo(dateStr: string): string {
  const seconds = Math.floor((Date.now() - new Date(dateStr).getTime()) / 1000);
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function Spinner() {
  return (
    <svg
      className="animate-spin h-3.5 w-3.5"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
  );
}

function ProgressBar({ value, max }: { value: number; max: number }) {
  const pct = max > 0 ? Math.round((value / max) * 100) : 0;
  return (
    <div className="w-full bg-gray-100 rounded-full h-1.5 mt-1.5">
      <div
        className="bg-gray-900 h-1.5 rounded-full transition-all duration-500"
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}

function StatusBadge({ doc }: { doc: Document }) {
  const isWorking = doc.status === 'uploaded' || doc.status === 'processing';

  return (
    <div className="flex flex-col items-end gap-1 min-w-[120px]">
      <span
        className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded-lg text-xs font-medium ${
          STATUS_COLORS[doc.status] || ''
        }`}
      >
        {isWorking && <Spinner />}
        {doc.status === 'uploaded' && 'Queued'}
        {doc.status === 'processing' && 'Indexing'}
        {doc.status === 'indexed' && 'Ready'}
        {doc.status === 'failed' && 'Failed'}
      </span>
      {doc.status === 'processing' && doc.page_count != null && doc.page_count > 0 && (
        <div className="w-full">
          <p className="text-[10px] text-gray-400 text-right">
            {doc.pages_indexed ?? 0}/{doc.page_count} pages
          </p>
          <ProgressBar value={doc.pages_indexed ?? 0} max={doc.page_count} />
        </div>
      )}
    </div>
  );
}

export function DocumentList({ isAdmin }: { isAdmin: boolean }) {
  const [docs, setDocs] = useState<Document[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    try {
      const data = await listDocuments();
      setDocs(data);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    const hasActive = docs.some(
      (d) => d.status === 'uploaded' || d.status === 'processing'
    );
    const interval = setInterval(load, hasActive ? 2000 : 10000);
    return () => clearInterval(interval);
  }, [docs.map((d) => d.status).join(',')]);

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this document?')) return;
    await deleteDocument(id);
    load();
  };

  if (loading) {
    return <p className="text-gray-400 text-center py-8">Loading documents...</p>;
  }

  if (docs.length === 0) {
    return (
      <p className="text-gray-400 text-center py-8">
        No documents yet. Upload one above.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      <h2 className="text-sm font-medium text-gray-500 uppercase tracking-wider">
        Documents ({docs.length})
      </h2>
      <div className="bg-white rounded-2xl shadow-sm overflow-hidden">
        {docs.map((doc, i) => (
          <div
            key={doc.id}
            className={`flex items-center gap-4 px-5 py-4 hover:bg-gray-50 transition-colors ${
              i > 0 ? 'border-t border-gray-50' : ''
            }`}
          >
            <div className="flex-1 min-w-0">
              <p className="text-sm text-gray-900 font-medium truncate">{doc.filename}</p>
              <p className="text-xs text-gray-400 mt-0.5">
                {formatBytes(doc.size_bytes)}
                {doc.page_count != null && ` · ${doc.page_count} pages`}
                {' · '}
                {timeAgo(doc.created_at)}
              </p>
              {doc.error_msg && (
                <p className="text-xs text-red-500 mt-1 truncate">{doc.error_msg}</p>
              )}
            </div>

            <StatusBadge doc={doc} />

            {isAdmin && (
              <button
                onClick={() => handleDelete(doc.id)}
                className="text-gray-300 hover:text-red-500 transition-colors text-sm"
                title="Delete"
              >
                ✕
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
