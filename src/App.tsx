import { useEffect } from "react";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { TranscriptView } from "./components/Transcript/TranscriptView";
import { ActionBar } from "./components/ActionBar/ActionBar";
import { useAppStore } from "./store/useAppStore";

function App() {
  const loadSessions = useAppStore((s) => s.loadSessions);
  const refreshActiveSessions = useAppStore((s) => s.refreshActiveSessions);
  const selectedSessionId = useAppStore((s) => s.selectedSessionId);

  useEffect(() => {
    loadSessions();
    const interval = setInterval(refreshActiveSessions, 5000);
    return () => clearInterval(interval);
  }, [loadSessions, refreshActiveSessions]);

  return (
    <div className="flex h-screen bg-bg-primary">
      <Sidebar />
      <main className="flex-1 flex flex-col min-w-0">
        {selectedSessionId ? (
          <>
            <TranscriptView />
            <ActionBar />
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <h2 className="text-text-secondary text-lg">Orbit</h2>
              <p className="text-text-muted text-sm mt-1">
                Select a session to view its transcript
              </p>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
