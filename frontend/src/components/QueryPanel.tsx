import { useState, useRef, useEffect } from 'react';
import { queryKnowledgebase, type QueryResponse } from '../api';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  meta?: QueryResponse;
}

export function QueryPanel() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const question = input.trim();
    if (!question || loading) return;

    setInput('');
    setMessages((prev) => [...prev, { role: 'user', content: question }]);
    setLoading(true);

    try {
      const response = await queryKnowledgebase(question);
      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: response.answer, meta: response },
      ]);
    } catch (e: any) {
      setMessages((prev) => [
        ...prev,
        { role: 'assistant', content: `Error: ${e.message || 'Query failed'}` },
      ]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-[calc(100vh-10rem)]">
      {/* Messages */}
      <div className="flex-1 overflow-y-auto space-y-4 pb-4">
        {messages.length === 0 && (
          <div className="text-center py-20">
            <p className="text-slate-500 text-lg">Ask a question about your documents</p>
            <p className="text-slate-600 text-sm mt-2">
              The agent will search through indexed documents using PageIndex tree navigation
            </p>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i}>
            <div
              className={`rounded-xl px-5 py-4 max-w-3xl ${
                msg.role === 'user'
                  ? 'bg-blue-600/20 border border-blue-500/30 ml-auto'
                  : 'bg-slate-800 border border-slate-700'
              }`}
            >
              <p className="text-xs font-medium mb-2 text-slate-400">
                {msg.role === 'user' ? 'You' : 'Agent'}
              </p>
              <div className="text-slate-200 whitespace-pre-wrap leading-relaxed">
                {msg.content}
              </div>
            </div>

            {/* Reasoning trace */}
            {msg.meta && msg.meta.reasoning_path.length > 0 && (
              <details className="mt-2 max-w-3xl">
                <summary className="text-xs text-slate-500 cursor-pointer hover:text-slate-400">
                  Reasoning path ({msg.meta.tools_used.length} tool calls)
                </summary>
                <div className="mt-2 bg-slate-800/50 border border-slate-700 rounded-lg p-3 text-xs text-slate-400 space-y-1">
                  {msg.meta.reasoning_path.map((step, j) => (
                    <p key={j} className="font-mono">
                      {j + 1}. {step}
                    </p>
                  ))}
                </div>
              </details>
            )}
          </div>
        ))}
        {loading && (
          <div className="bg-slate-800 border border-slate-700 rounded-xl px-5 py-4 max-w-3xl">
            <p className="text-xs font-medium mb-2 text-slate-400">Agent</p>
            <p className="text-slate-400 animate-pulse">Thinking...</p>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex gap-3 pt-4 border-t border-slate-700">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Ask a question..."
          disabled={loading}
          className="flex-1 bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-white placeholder:text-slate-500 focus:outline-none focus:border-blue-500 transition-colors disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={loading || !input.trim()}
          className="bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-500 text-white px-6 py-3 rounded-xl font-medium transition-colors"
        >
          Send
        </button>
      </form>
    </div>
  );
}
