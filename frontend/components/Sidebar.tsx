import React from "react";

export interface Chat {
  id: string;
  title: string;
}

interface SidebarProps {
  chats: Chat[];
  onNewChat: () => void;
  onSelectChat: (chatId: string) => void;
  onDeleteChat: (chatId: string) => void;
  selectedChatId: string | null;
}

export const Sidebar: React.FC<SidebarProps> = ({ chats, onNewChat, onSelectChat, onDeleteChat, selectedChatId }) => {
  return (
    <aside className="h-screen w-64 bg-[#f8f5ee] border-r border-gray-200 flex flex-col p-4">
      <button
        className="mb-6 flex items-center justify-center bg-green-200 hover:bg-green-300 text-green-900 font-bold py-2 px-4 rounded transition"
        onClick={onNewChat}
      >
        + New Chat
      </button>
      <div className="flex-1 overflow-y-auto">
        <h2 className="text-xs font-semibold text-gray-500 mb-2">Previous Chats</h2>
        <ul className="space-y-2">
          {chats.map((chat) => (
            <li key={chat.id} className="flex items-center group">
              <button
                className={`flex-1 text-left px-3 py-2 rounded-lg transition font-medium ${
                  selectedChatId === chat.id
                    ? "bg-green-100 text-green-900"
                    : "bg-white text-gray-800 hover:bg-green-50"
                }`}
                onClick={() => onSelectChat(chat.id)}
              >
                {chat.title}
              </button>
              <button
                className="ml-2 p-1 rounded hover:bg-red-100 text-gray-400 hover:text-red-600 transition-opacity opacity-0 group-hover:opacity-100"
                title="Delete chat"
                onClick={() => onDeleteChat(chat.id)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
                  <path fillRule="evenodd" d="M7.293 7.293a1 1 0 011.414 0L10 8.586l1.293-1.293a1 1 0 111.414 1.414L11.414 10l1.293 1.293a1 1 0 01-1.414 1.414L10 11.414l-1.293 1.293a1 1 0 01-1.414-1.414L8.586 10 7.293 8.707a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}; 