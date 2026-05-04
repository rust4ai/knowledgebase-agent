import { useCallback, useState } from 'react';
import { uploadDocument } from '../api';

export function Upload({ onUploaded }: { onUploaded: () => void }) {
  const [dragging, setDragging] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFiles = useCallback(
    async (files: FileList | null) => {
      if (!files?.length) return;
      setError(null);
      setUploading(true);
      try {
        for (const file of Array.from(files)) {
          await uploadDocument(file);
        }
        onUploaded();
      } catch (e: any) {
        setError(e.message || 'Upload failed');
      } finally {
        setUploading(false);
      }
    },
    [onUploaded]
  );

  return (
    <div
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        handleFiles(e.dataTransfer.files);
      }}
      className={`border-2 border-dashed rounded-xl p-10 text-center transition-colors ${
        dragging
          ? 'border-blue-400 bg-blue-500/10'
          : 'border-slate-600 hover:border-slate-500'
      }`}
    >
      {uploading ? (
        <p className="text-slate-400">Uploading...</p>
      ) : (
        <>
          <p className="text-slate-400 mb-3">
            Drag & drop files here, or{' '}
            <label className="text-blue-400 hover:text-blue-300 cursor-pointer underline">
              browse
              <input
                type="file"
                multiple
                className="hidden"
                onChange={(e) => handleFiles(e.target.files)}
              />
            </label>
          </p>
          <p className="text-xs text-slate-500">
            Supports .txt, .md, .pdf, .json, and more
          </p>
        </>
      )}
      {error && <p className="text-red-400 text-sm mt-3">{error}</p>}
    </div>
  );
}
