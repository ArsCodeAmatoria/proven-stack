/** Offline sync primitives — foundation stub (no domain rules). */

export type OutboxItem = {
  mutationId: string;
  op: string;
  body: unknown;
  state: "pending" | "in_flight" | "acked" | "dead" | "conflict";
};

export type SyncEngine = {
  enqueue(item: Omit<OutboxItem, "state">): Promise<void>;
  pendingCount(): Promise<number>;
};

export function createMemorySyncEngine(): SyncEngine {
  const items: OutboxItem[] = [];
  return {
    async enqueue(item) {
      items.push({ ...item, state: "pending" });
    },
    async pendingCount() {
      return items.filter((i) => i.state === "pending" || i.state === "in_flight")
        .length;
    },
  };
}
