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
      className={`border-2 border-dashed rounded-2xl p-10 text-center transition-colors bg-white ${
        dragging
          ? 'border-gray-900 bg-gray-50'
          : 'border-gray-200 hover:border-gray-300'
      }`}
    >
      {uploading ? (
        <p className="text-gray-400">Uploading...</p>
      ) : (
        <>
          <p className="text-gray-500 mb-3">
            Drag & drop files here, or{' '}
            <label className="text-gray-900 font-medium hover:underline cursor-pointer">
              browse
              <input
                type="file"
                multiple
                className="hidden"
                onChange={(e) => handleFiles(e.target.files)}
              />
            </label>
          </p>
          <p className="text-xs text-gray-400">
            Supports .txt, .md, .pdf, .json, and more
          </p>
        </>
      )}
      {error && <p className="text-red-500 text-sm mt-3">{error}</p>}
    </div>
  );
}
