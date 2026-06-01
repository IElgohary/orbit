import { useRef, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useAppStore } from "../../store/useAppStore";
import { MessageBubble } from "./MessageBubble";

export function TranscriptView() {
  const messages = useAppStore((s) => s.messages);
  const selectedSessionId = useAppStore((s) => s.selectedSessionId);
  const loading = useAppStore((s) => s.loading);

  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 120,
    overscan: 5,
  });

  useEffect(() => {
    if (messages.length > 0 && parentRef.current) {
      parentRef.current.scrollTop = parentRef.current.scrollHeight;
    }
  }, [selectedSessionId]);

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <span className="text-text-muted text-sm">Loading transcript...</span>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className="flex-1 overflow-y-auto"
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const message = messages[virtualItem.index];
          return (
            <div
              key={virtualItem.key}
              data-index={virtualItem.index}
              ref={virtualizer.measureElement}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <div className="max-w-3xl mx-auto px-6 py-2">
                <MessageBubble message={message} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
