const BASE = '/api';

export type Document = {
  id: string;
  filename: string;
  mime_type: string;
  s3_key: string;
  size_bytes: number;
  status: 'uploaded' | 'processing' | 'indexed' | 'failed';
  page_count: number | null;
  pages_indexed: number | null;
  error_msg: string | null;
  created_at: string;
  updated_at: string;
};

export type QueryResponse = {
  answer: string;
  reasoning_path: string[];
  tools_used: string[];
};

export async function uploadDocument(file: File): Promise<Document> {
  const form = new FormData();
  form.append('file', file);
  const res = await fetch(`${BASE}/documents`, { method: 'POST', body: form });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function listDocuments(): Promise<Document[]> {
  const res = await fetch(`${BASE}/documents`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function getDocument(id: string): Promise<Document> {
  const res = await fetch(`${BASE}/documents/${id}`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function deleteDocument(id: string): Promise<void> {
  const res = await fetch(`${BASE}/documents/${id}`, { method: 'DELETE' });
  if (!res.ok) throw new Error(await res.text());
}

export async function queryKnowledgebase(question: string): Promise<QueryResponse> {
  const res = await fetch(`${BASE}/query`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question }),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}
