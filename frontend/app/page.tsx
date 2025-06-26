'use client';

import { useState } from "react";
import { SearchBar } from "@/components/SearchBar";
import { AnswerBox } from "@/components/AnswerBox";
import { ChunkViewer } from "@/components/ChunkViewer";

export default function Home() {
  // Removed unused query state
  const [answer, setAnswer] = useState("");
  const [chunks, setChunks] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const handleSearch = async (input: string) => {
    setLoading(true);
    try {
      const res = await fetch("/api/rag", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: input }),
      });
      const data = await res.json();
      setAnswer(data.answer);
      setChunks(data.chunks);
    } catch (err) {
      console.error("Failed to fetch answer", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="p-8 space-y-6 max-w-3xl mx-auto">
      <h1 className="text-3xl font-bold">🧠 Garden Intel Patent RAG</h1>
      <SearchBar onSearch={handleSearch} loading={loading} />
      <AnswerBox answer={answer} />
      <ChunkViewer chunks={chunks} />
    </main>
  );
}
