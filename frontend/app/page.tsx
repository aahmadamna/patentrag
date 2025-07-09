'use client';

import { useState } from "react";
// import { v4 as uuidv4 } from 'uuid'; // Commented out because 'uuid' module not found

interface SearchResult {
  patent_id: string;
  chunk_id: string;
  snippet: string;
  distance: number;
}

interface Chat {
  id: string;
  title: string;
}

interface SearchBarProps {
  onSearch: (input: string, mode: 'ask' | 'keyword') => void;
  onFileUpload: (file: File) => void;
  uploadedFileName: string | null;
  loading: boolean;
  mode: 'ask' | 'keyword';
  setMode: (mode: 'ask' | 'keyword') => void;
}

interface AnswerBoxProps {
  answer: string;
}

interface ChunkViewerProps {
  chunks: string[];
}

const SearchBar: React.FC<SearchBarProps> = ({ onSearch, onFileUpload, uploadedFileName, loading, mode, setMode }) => {
  const [input, setInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      onSearch(input, mode);
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      onFileUpload(e.target.files[0]);
    }
  };

  return (
    <div className="w-full space-y-4">
      {!uploadedFileName ? (
        <label className="flex items-center justify-center w-full p-3 bg-green-500 text-white rounded-full cursor-pointer hover:bg-green-600 transition">
          <span className="text-2xl mr-2">+</span> Upload PDF Patent
          <input type="file" accept=".pdf" className="hidden" onChange={handleFileChange} />
        </label>
      ) : (
        <div className="w-full p-3 bg-gray-100 text-gray-800 rounded-full text-center">
          Uploaded: {uploadedFileName}
        </div>
      )}
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={mode === 'ask' ? "Ask a question about the patent..." : "Enter keywords for search..."}
          className="flex-1 p-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 transition"
          disabled={loading}
        />
        <button
          onClick={handleSubmit}
          className={`px-4 py-3 rounded-lg text-white font-semibold ${
            loading ? "bg-gray-400 cursor-not-allowed" : "bg-blue-600 hover:bg-blue-700"
          } transition`}
          disabled={loading}
        >
          {loading ? "Processing..." : "Submit"}
        </button>
      </div>
      <div className="flex justify-center gap-4">
        <button
          onClick={() => setMode('ask')}
          className={`px-4 py-2 rounded-lg font-semibold ${
            mode === 'ask' ? "bg-blue-600 text-white" : "bg-gray-200 text-gray-800"
          } transition`}
        >
          Ask
        </button>
        <button
          onClick={() => setMode('keyword')}
          className={`px-4 py-2 rounded-lg font-semibold ${
            mode === 'keyword' ? "bg-blue-600 text-white" : "bg-gray-200 text-gray-800"
          } transition`}
        >
          Keyword Search
        </button>
      </div>
    </div>
  );
};

const AnswerBox: React.FC<AnswerBoxProps> = ({ answer }) => {
  return (
    <div className="bg-white p-6 rounded-lg shadow-md">
      <h2 className="text-xl font-semibold mb-4 text-gray-800">Response</h2>
      <p className="text-gray-600">{answer || "No response yet. Try submitting a query!"}</p>
    </div>
  );
};

const ChunkViewer: React.FC<ChunkViewerProps> = ({ chunks }) => {
  return (
    <div className="bg-white p-6 rounded-lg shadow-md">
      <h2 className="text-xl font-semibold mb-4 text-gray-800">Relevant Chunks</h2>
      {chunks.length > 0 ? (
        <ul className="space-y-4">
          {chunks.map((chunk, index) => (
            <li key={index} className="p-4 bg-gray-50 rounded-md text-gray-700">
              {chunk}
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-gray-600">No chunks to display.</p>
      )}
    </div>
  );
};

export default function Home() {
  const [answer, setAnswer] = useState("");
  const [chunks, setChunks] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [mode, setMode] = useState<'ask' | 'keyword'>('ask');
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);
  const [patentId, setPatentId] = useState<string | null>(null);
  const [chats, setChats] = useState<Chat[]>([]);
  const [currentChatId, setCurrentChatId] = useState<string | null>(null);

  const handleNewChat = () =>
    {
      const newChat: Chat = { id: crypto.randomUUID(), title: `Chat ${chats.length + 1}` };
      setChats([...chats, newChat]);
      setCurrentChatId(newChat.id);
      setAnswer("");
      setChunks([]);
    setUploadedFileName(null);
  };

  const handleSelectChat = (chatId: string) => {
    setCurrentChatId(chatId);
    setAnswer("");
    setChunks([]);
    setUploadedFileName(null);
  };

  const handleFileUpload = async (file: File) => {
    setUploadedFileName(file.name);
    setLoading(true);
    try {
      const formData = new FormData();
      formData.append('file', file);
      const res = await fetch('http://localhost:8000/upload_pdf', {
        method: 'POST',
        body: formData,
      });
      if (!res.ok) {
        throw new Error('Failed to upload PDF');
      }
      const data = await res.json();
      if (data.patent_id) {
        setPatentId(data.patent_id);
      }
    } catch (err) {
      console.error('PDF upload failed', err);
      setAnswer('Error uploading PDF. Please try again.');
      setUploadedFileName(null);
      setPatentId(null);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async (input: string, searchMode: 'ask' | 'keyword') => {
    setLoading(true);
    try {
      const endpoint = searchMode === 'ask' ? 'http://localhost:8000/query' : 'http://localhost:8000/search';
      const payload = searchMode === 'ask'
        ? { question: input, patent_id: patentId }
        : { query: input, patent_id: patentId };
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      
      if (searchMode === 'ask') {
        setAnswer(data.answer || "");
        setChunks([]);
      } else {
        // For keyword search, data is an array of search results
        setAnswer("");
        setChunks(Array.isArray(data) ? data.map((result: SearchResult) => result.snippet) : []);
      }
    } catch (err) {
      console.error("Failed to fetch response", err);
      setAnswer("Error fetching response. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-gray-100 flex">
      {/* Sidebar */}
      <div className="w-64 bg-white shadow-md p-4 flex flex-col">
        <button
          onClick={handleNewChat}
          className="flex items-center justify-center p-2 mb-4 bg-blue-600 text-white rounded-full hover:bg-blue-700 transition"
        >
          <span className="text-xl">+</span> New Chat
        </button>
        <div className="flex-1 overflow-y-auto">
          {chats.map((chat) => (
            <button
              key={chat.id}
              onClick={() => handleSelectChat(chat.id)}
              className={`w-full text-left p-2 rounded-lg mb-2 ${
                currentChatId === chat.id ? "bg-blue-100 text-blue-800" : "bg-gray-100 text-gray-800"
              } hover:bg-blue-50 transition`}
            >
              {chat.title}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 p-4 md:p-8 flex flex-col items-center">
        <div className="w-full max-w-4xl space-y-6">
          <h1 className="text-3xl md:text-4xl font-bold text-center text-gray-800">
            🧠 Garden Intel Patent RAG
          </h1>
          <SearchBar
            onSearch={handleSearch}
            onFileUpload={handleFileUpload}
            uploadedFileName={uploadedFileName}
            loading={loading}
            mode={mode}
            setMode={setMode}
          />
          <AnswerBox answer={answer} />
          {mode === 'keyword' && <ChunkViewer chunks={chunks} />}
        </div>
      </div>
    </main>
  );
}