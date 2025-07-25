-- Create chats table
CREATE TABLE IF NOT EXISTS chats (
    id UUID PRIMARY KEY,
    title TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    sender TEXT, -- 'user' or 'ai'
    content TEXT,
    type TEXT,   -- 'question', 'answer', etc.
    created_at TIMESTAMP DEFAULT NOW()
); 