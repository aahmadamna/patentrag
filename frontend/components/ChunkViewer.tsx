interface Props {
    chunks: string[];
  }
  
  export function ChunkViewer({ chunks }: Props) {
    if (!chunks || chunks.length === 0) return null;
  
    return (
      <div>
        <h2 className="text-xl font-semibold mt-6">📄 Retrieved Chunks</h2>
        <ul className="list-disc pl-5 space-y-2 mt-2">
          {chunks.map((chunk, i) => (
            <li key={i} className="bg-gray-100 p-3 rounded">
              {chunk}
            </li>
          ))}
        </ul>
      </div>
    );
  }
  