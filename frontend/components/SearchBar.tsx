'use client';

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useState } from "react";

function renderResult(result: unknown): string {
  if (typeof result === 'string') return result;
  if (typeof result === 'object' && result !== null) return JSON.stringify(result, null, 2);
  return String(result);
}

export function SearchBar() {
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<unknown>(null);
  const [mode, setMode] = useState<'search' | 'rag'>('search');

  const handleSubmit = async () => {
    setLoading(true);
    setResult(null);
    try {
      const endpoint = mode === 'search' ? '/api/search' : '/api/rag';
      const body = mode === 'search' 
        ? { query: input }
        : { question: input };
      
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      setResult(data);
    } catch {
      setResult({ error: "Failed to fetch result." });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex gap-2">
        <Input
          placeholder={mode === 'search' ? "Search for patent chunks..." : "Ask a question about the patent..."}
          value={input}
          onChange={(e) => setInput(e.target.value)}
        />
        <Button onClick={handleSubmit} disabled={loading}>
          {loading ? "Thinking..." : mode === 'search' ? "Search" : "Ask"}
        </Button>
        <Button 
          className="bg-gray-100 text-gray-900 hover:bg-gray-200"
          onClick={() => setMode(mode === 'search' ? 'rag' : 'search')}
          disabled={loading}
        >
          {mode === 'search' ? 'Q&A Mode' : 'Search Mode'}
        </Button>
      </div>
      {result !== null && result !== undefined && (
        <pre className="bg-gray-100 p-2 rounded text-sm mt-2">
          {renderResult(result)}
        </pre>
      )}
    </div>
  );
}
