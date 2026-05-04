import { useState, useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
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
            <p className="text-gray-400 text-lg">Ask a question about your documents</p>
            <p className="text-gray-300 text-sm mt-2">
              The agent will search through indexed documents using PageIndex tree navigation
            </p>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i}>
            <div
              className={`rounded-2xl px-5 py-4 max-w-3xl ${
                msg.role === 'user'
                  ? 'bg-gray-900 text-white ml-auto'
                  : 'bg-white shadow-sm'
              }`}
            >
              <p className={`text-xs font-medium mb-2 ${
                msg.role === 'user' ? 'text-gray-400' : 'text-gray-400'
              }`}>
                {msg.role === 'user' ? 'You' : 'Agent'}
              </p>
              {msg.role === 'user' ? (
                <div className="text-white text-sm leading-relaxed">
                  {msg.content}
                </div>
              ) : (
                <div className="prose prose-sm prose-gray max-w-none">
                  <ReactMarkdown>{msg.content}</ReactMarkdown>
                </div>
              )}
            </div>

            {msg.meta && msg.meta.reasoning_path.length > 0 && (
              <details className="mt-2 max-w-3xl">
                <summary className="text-xs text-gray-400 cursor-pointer hover:text-gray-600">
                  Reasoning path ({msg.meta.tools_used.length} tool calls)
                </summary>
                <div className="mt-2 bg-gray-50 rounded-xl p-3 text-xs text-gray-500 space-y-1">
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
          <div className="bg-white shadow-sm rounded-2xl px-5 py-4 max-w-3xl">
            <p className="text-xs font-medium mb-2 text-gray-400">Agent</p>
            <p className="text-gray-400 animate-pulse text-sm">Thinking...</p>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex gap-3 pt-4 border-t border-gray-200">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Ask a question..."
          disabled={loading}
          className="flex-1 bg-gray-100 rounded-xl px-4 py-3 text-sm text-gray-900 placeholder:text-gray-400 outline-none focus:ring-2 focus:ring-gray-200 transition-all disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={loading || !input.trim()}
          className="bg-gray-900 hover:bg-gray-800 disabled:bg-gray-200 disabled:text-gray-400 text-white px-6 py-3 rounded-xl text-sm font-medium transition-colors"
        >
          Send
        </button>
      </form>
    </div>
  );
}
