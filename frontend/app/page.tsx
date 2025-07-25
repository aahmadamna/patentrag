'use client';

import { useState, useEffect } from "react";
import { Sidebar } from "@/components/Sidebar";

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

interface Message {
  id: string;
  sender: string;
  content: string;
  msg_type: string;
  created_at: string;
}

interface AnswerBoxProps {
  messages: Message[];
}

interface ChunkViewerProps {
  chunks: string[];
  searchQuery?: string;
}

interface RelatedDocument {
  patent_id: string;
  snippet: string;
  similarity: number;
}

// Helper function to highlight keywords in text
const highlightKeywords = (text: string, keywords: string): string => {
  if (!keywords.trim()) return text;
  
  const escapedKeywords = keywords.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedKeywords})`, 'gi');
  
  return text.replace(regex, '<mark class="bg-yellow-200 px-1 rounded">$1</mark>');
};

// Helper function to export search results
const exportSearchResults = (chunks: string[], searchQuery: string) => {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `search-results-${searchQuery}-${timestamp}.txt`;
  
  let content = `Search Results Export\n`;
  content += `===================\n\n`;
  content += `Search Query: ${searchQuery}\n`;
  content += `Timestamp: ${new Date().toLocaleString()}\n`;
  content += `Total Results: ${chunks.length}\n\n`;
  content += `Results:\n`;
  content += `========\n\n`;
  
  chunks.forEach((chunk, index) => {
    content += `Result ${index + 1}:\n`;
    content += `${chunk}\n\n`;
    content += `---\n\n`;
  });
  
  const blob = new Blob([content], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

const AnswerBox: React.FC<AnswerBoxProps> = ({ messages }) => {
  return (
    <div className="bg-white p-6 rounded-lg shadow-md max-h-96 overflow-y-auto">
      <h2 className="text-xl font-semibold mb-4 text-gray-800">Conversation History</h2>
      {messages.length > 0 ? (
        <div className="space-y-4">
          {messages.map((message, index) => (
            <div key={index} className={`p-3 rounded-lg ${
              message.sender === 'user' 
                ? 'bg-blue-100 ml-8' 
                : 'bg-gray-100 mr-8'
            }`}>
              <div className="font-semibold text-sm text-gray-600 mb-1">
                {message.sender === 'user' ? 'You' : 'AI Assistant'}
              </div>
              <div className="text-gray-800">{message.content}</div>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-gray-600">No messages yet. Start a conversation!</p>
      )}
    </div>
  );
};

const ChunkViewer: React.FC<ChunkViewerProps> = ({ chunks, searchQuery = '' }) => {
  return (
    <div className="bg-white p-6 rounded-lg shadow-md">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-semibold text-gray-800">Relevant Chunks</h2>
        {chunks.length > 0 && searchQuery && (
          <button
            onClick={() => exportSearchResults(chunks, searchQuery)}
            className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition text-sm"
          >
            📄 Export Results
          </button>
        )}
      </div>
      {chunks.length > 0 ? (
        <ul className="space-y-4">
          {chunks.map((chunk, index) => (
            <li key={index} className="p-4 bg-gray-50 rounded-md text-gray-700">
              <div 
                className="whitespace-pre-wrap"
                dangerouslySetInnerHTML={{ 
                  __html: highlightKeywords(chunk, searchQuery) 
                }}
              />
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
  const [messages, setMessages] = useState<Message[]>([]);
  const [chunks, setChunks] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [mode, setMode] = useState<'ask' | 'keyword'>('ask');
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);
  const [patentId, setPatentId] = useState<string | null>(null);
  const [chats, setChats] = useState<Chat[]>([]);
  const [currentChatId, setCurrentChatId] = useState<string | null>(null);
  const [lastSearchQuery, setLastSearchQuery] = useState<string>('');
  const [summary, setSummary] = useState<string>('');
  const [summaryLoading, setSummaryLoading] = useState(false);
  const [relatedDocs, setRelatedDocs] = useState<RelatedDocument[]>([]);
  const [relatedDocsLoading, setRelatedDocsLoading] = useState(false);

  // Load chats on component mount
  useEffect(() => {
    loadChats();
  }, []);

  // Load messages when chat changes
  useEffect(() => {
    if (currentChatId) {
      loadMessages(currentChatId);
    } else {
      setMessages([]);
    }
  }, [currentChatId]);

  const loadChats = async () => {
    try {
      const res = await fetch('http://localhost:8000/chats');
      if (res.ok) {
        const data = await res.json();
        setChats(data);
      }
    } catch (err) {
      console.error('Failed to load chats:', err);
    }
  };

  const loadMessages = async (chatId: string) => {
    try {
      const res = await fetch(`http://localhost:8000/chats/${chatId}/messages`);
      if (res.ok) {
        const data = await res.json();
        setMessages(data);
      }
    } catch (err) {
      console.error('Failed to load messages:', err);
    }
  };

  const handleNewChat = async () => {
    setLoading(true);
    try {
      const chatTitle = uploadedFileName ? uploadedFileName.replace('.pdf', '') : `Chat ${chats.length + 1}`;
      const res = await fetch('http://localhost:8000/chats', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: chatTitle }),
      });
      if (res.ok) {
        const newChat = await res.json();
        setChats([...chats, newChat]);
        setCurrentChatId(newChat.id);
        setMessages([]);
        setChunks([]);
        setUploadedFileName(null);
        setPatentId(null);
      }
    } catch (err) {
      console.error('Failed to create chat:', err);
    } finally {
      setLoading(false);
    }
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
      setUploadedFileName(null);
      setPatentId(null);
    } finally {
      setLoading(false);
    }
  };

  const saveMessage = async (sender: string, content: string, msgType: string) => {
    if (!currentChatId) return;
    
    try {
      const res = await fetch(`http://localhost:8000/chats/${currentChatId}/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sender, content, msg_type: msgType }),
      });
      if (res.ok) {
        const newMessage = await res.json();
        setMessages([...messages, newMessage]);
      }
    } catch (err) {
      console.error('Failed to save message:', err);
    }
  };

  const handleSearch = async (input: string, searchMode: 'ask' | 'keyword') => {
    if (!currentChatId) {
      // Create a new chat if none exists
      await handleNewChat();
      // Wait a bit for the chat to be created
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    setLoading(true);
    
    // Save user message
    await saveMessage('user', input, searchMode === 'ask' ? 'question' : 'search');
    
    try {
      const endpoint = searchMode === 'ask' ? 'http://localhost:8000/query' : 'http://localhost:8000/search';
      const payload = searchMode === 'ask'
        ? { question: input, patent_id: patentId }
        : { query: input, patent_id: patentId, search_mode: 'keyword' };
      
      console.log('🔍 Frontend sending request:', { endpoint, payload, searchMode });
      
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      
      if (searchMode === 'ask') {
        // Save AI response
        await saveMessage('ai', data.answer || "No answer available", 'answer');
        setChunks([]);
      } else {
        // For keyword search, save the results as a message
        const searchResults = Array.isArray(data) ? data.map((result: SearchResult) => result.snippet).join('\n\n') : 'No results found';
        await saveMessage('ai', `Search results:\n${searchResults}`, 'search_results');
        setChunks(Array.isArray(data) ? data.map((result: SearchResult) => result.snippet) : []);
      }
      setLastSearchQuery(input);
    } catch (err) {
      console.error("Failed to fetch response", err);
      await saveMessage('ai', "Error fetching response. Please try again.", 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteChat = async (chatId: string) => {
    try {
      const res = await fetch(`http://localhost:8000/chats/${chatId}`, { method: 'DELETE' });
      if (res.ok) {
        setChats(chats => chats.filter(chat => chat.id !== chatId));
        if (currentChatId === chatId) {
          setCurrentChatId(null);
          setMessages([]);
          setChunks([]);
        }
      } else {
        console.error('Failed to delete chat');
      }
    } catch (err) {
      console.error('Failed to delete chat:', err);
    }
  };

  const generateSummary = async () => {
    if (!patentId) return;
    
    setSummaryLoading(true);
    try {
      const res = await fetch('http://localhost:8000/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          question: "Please provide a comprehensive summary of this document, including its main topics, key findings, and important details.",
          patent_id: patentId,
          top_k: 10
        }),
      });
      const data = await res.json();
      setSummary(data.answer);
    } catch (err) {
      console.error('Failed to generate summary:', err);
      setSummary('Failed to generate summary. Please try again.');
    } finally {
      setSummaryLoading(false);
    }
  };

  const findRelatedDocuments = async () => {
    if (!patentId) return;
    
    setRelatedDocsLoading(true);
    try {
      const res = await fetch(`http://localhost:8000/related-documents/${patentId}`);
      if (res.ok) {
        const data = await res.json();
        setRelatedDocs(data);
      } else {
        console.error('Failed to find related documents');
      }
    } catch (err) {
      console.error('Failed to find related documents:', err);
    } finally {
      setRelatedDocsLoading(false);
    }
  };

  const downloadSummaryPDF = async () => {
    if (!patentId || !summary || !uploadedFileName) return;
    
    try {
      const res = await fetch('http://localhost:8000/download-summary-pdf', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          patent_id: patentId,
          summary: summary,
          filename: uploadedFileName.replace('.pdf', '')
        }),
      });
      
      if (res.ok) {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `smart-summary-${uploadedFileName.replace('.pdf', '')}.txt`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      } else {
        console.error('Failed to download summary');
      }
    } catch (err) {
      console.error('Failed to download summary:', err);
    }
  };

  return (
    <div className="flex min-h-screen bg-[#f8f5ee]">
      <Sidebar
        chats={chats}
        onNewChat={handleNewChat}
        onSelectChat={chatId => {
          setCurrentChatId(chatId);
          setChunks([]);
          setUploadedFileName(null);
          setPatentId(null);
          setSummary(''); // Clear summary on chat change
          setRelatedDocs([]); // Clear related docs on chat change
        }}
        onDeleteChat={handleDeleteChat}
        selectedChatId={currentChatId}
      />
      <div className="flex flex-col flex-1 min-h-screen">
        <div className="flex-1 p-8 overflow-y-auto flex flex-col items-center">
          <div className="w-full max-w-2xl space-y-8">
            {/* Upload Patent */}
            {!uploadedFileName ? (
              <label className="flex items-center justify-center w-full p-4 bg-[#8fc490] text-white rounded-lg cursor-pointer hover:bg-[#7bb37e] transition font-semibold text-lg">
                <span className="text-2xl mr-2">+</span> Upload PDF Patent
                <input type="file" accept=".pdf" className="hidden" onChange={e => { if (e.target.files && e.target.files[0]) handleFileUpload(e.target.files[0]); }} disabled={loading} />
              </label>
            ) : (
              <div className="space-y-4">
                <div className="w-full p-4 bg-white text-green-900 rounded-lg text-center font-semibold border border-green-200">
                  Uploaded: {uploadedFileName}
                </div>
                <div className="flex justify-center gap-4">
                  <button
                    onClick={generateSummary}
                    disabled={summaryLoading}
                    className={`px-6 py-3 rounded-lg text-white font-semibold transition ${
                      summaryLoading ? "bg-gray-400 cursor-not-allowed" : "bg-[#8fc490] hover:bg-[#7bb37e]"
                    }`}
                  >
                    {summaryLoading ? "Generating Summary..." : "📋 Generate Smart Summary"}
                  </button>
                  {summary && (
                    <button
                      onClick={downloadSummaryPDF}
                      className="px-6 py-3 rounded-lg text-white font-semibold transition bg-[#8fc490] hover:bg-[#7bb37e]"
                    >
                      📥 Download Summary PDF
                    </button>
                  )}
                </div>
                {summary && (
                  <div className="bg-white p-6 rounded-lg shadow-md border border-green-200">
                    <h3 className="text-lg font-semibold mb-3 text-gray-800">📋 Document Summary</h3>
                    <div className="text-gray-700 whitespace-pre-wrap">{summary}</div>
                  </div>
                )}
                <div className="flex justify-center">
                  <button
                    onClick={findRelatedDocuments}
                    disabled={relatedDocsLoading}
                    className={`px-6 py-3 rounded-lg text-white font-semibold transition ${
                      relatedDocsLoading ? "bg-gray-400 cursor-not-allowed" : "bg-[#8fc490] hover:bg-[#7bb37e]"
                    }`}
                  >
                    {relatedDocsLoading ? "Finding Related Documents..." : "🔗 Find Related Documents"}
                  </button>
                </div>
                {relatedDocs.length > 0 && (
                  <div className="bg-white p-6 rounded-lg shadow-md border border-green-200">
                    <h3 className="text-lg font-semibold mb-3 text-gray-800">🔗 Related Documents</h3>
                    <div className="space-y-3">
                      {relatedDocs.map((doc, index) => (
                        <div key={index} className="p-3 bg-gray-50 rounded-md">
                          <div className="text-sm text-gray-600 mb-1">
                            Patent ID: {doc.patent_id} • Similarity: {(doc.similarity * 100).toFixed(1)}%
                          </div>
                          <div className="text-gray-800 text-sm">{doc.snippet.substring(0, 200)}...</div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Question Input & Toggle */}
            <form
              onSubmit={e => {
                e.preventDefault();
                const input = (e.target as HTMLFormElement).elements.namedItem('input') as HTMLInputElement;
                if (input && input.value.trim()) {
                  handleSearch(input.value, mode);
                  input.value = '';
                }
              }}
              className="flex gap-2 items-center"
            >
              <input
                name="input"
                type="text"
                placeholder={mode === 'ask' ? "Ask a question about the patent..." : "Enter keywords for search..."}
                className="flex-1 p-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-[#8fc490] bg-white text-gray-900"
                disabled={loading}
              />
              <button
                type="submit"
                className={`px-6 py-3 rounded-lg text-white font-semibold transition ${loading ? "bg-gray-400 cursor-not-allowed" : "bg-[#8fc490] hover:bg-[#7bb37e]"}`}
                disabled={loading}
              >
                {loading ? "Processing..." : mode === 'ask' ? "Ask" : "Search"}
              </button>
            </form>
            <div className="flex justify-center gap-4">
              <button
                type="button"
                onClick={() => setMode('ask')}
                className={`px-4 py-2 rounded-lg font-semibold transition border ${mode === 'ask' ? "bg-[#8fc490] text-white border-[#8fc490]" : "bg-white text-green-900 border-green-200 hover:bg-green-50"}`}
                disabled={loading}
              >
                Q&A Mode
              </button>
              <button
                type="button"
                onClick={() => setMode('keyword')}
                className={`px-4 py-2 rounded-lg font-semibold transition border ${mode === 'keyword' ? "bg-[#8fc490] text-white border-[#8fc490]" : "bg-white text-green-900 border-green-200 hover:bg-green-50"}`}
                disabled={loading}
              >
                Keyword Search
              </button>
            </div>

            {/* Results */}
            {mode === 'ask' ? (
              <AnswerBox messages={messages} />
            ) : (
              <ChunkViewer chunks={chunks} searchQuery={lastSearchQuery} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}