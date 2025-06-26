'use client';

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useState } from "react";

interface Props {
  onSearch: (query: string) => void;
  loading?: boolean;
}

export const SearchBar = ({ onSearch, loading }: Props) => {
  const [input, setInput] = useState("");

  const handleSubmit = () => {
    onSearch(input);
  };

  return (
    <div className="flex gap-2">
      <Input
        placeholder="Ask a question about a patent..."
        value={input}
        onChange={(e) => setInput(e.target.value)}
      />
      <Button onClick={handleSubmit} disabled={loading}>
        {loading ? "Thinking..." : "Ask"}
      </Button>
    </div>
  );
};
