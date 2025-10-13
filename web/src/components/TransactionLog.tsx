// TransactionLog component for detailed transaction log viewing

import { useState, useCallback, useEffect } from "preact/hooks";
import { apiClient } from "../services/api";

interface Transaction {
  id: string;
  timestamp: string;
  type: string;
  status: string;
  data: any;
  error?: string;
}

interface TransactionLogProps {
  benchmarkId: string | null;
  executionId: string | null;
  isRunning: boolean;
}

export function TransactionLog({ benchmarkId, executionId, isRunning }: TransactionLogProps) {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [autoRefresh, setAutoRefresh] = useState(true);

  // Load transaction logs
  const loadTransactions = useCallback(async () => {
    if (!benchmarkId || !executionId) return;

    setLoading(true);
    setError(null);

    try {
      // Mock transaction data for now - in real implementation this would call API
      const mockTransactions: Transaction[] = [
        {
          id: "1",
          timestamp: new Date().toISOString(),
          type: "instruction",
          status: "success",
          data: {
            program: "SystemProgram",
            instruction: "Transfer",
            accounts: ["source", "destination"],
            amount: 1000000
          }
        },
        {
          id: "2",
          timestamp: new Date(Date.now() - 1000).toISOString(),
          type: "transaction",
          status: "success",
          data: {
            signature: "5j7s83...",
            slot: 123456,
            blockTime: new Date().toISOString(),
            fee: 5000
          }
        },
        {
          id: "3",
          timestamp: new Date(Date.now() - 2000).toISOString(),
          type: "error",
          status: "failed",
          data: {
            instruction: "Transfer",
            error: "Insufficient funds"
          },
          error: "Transaction failed: Insufficient funds for transfer"
        }
      ];

      setTransactions(mockTransactions);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load transactions");
    } finally {
      setLoading(false);
    }
  }, [benchmarkId, executionId]);

  // Auto-refresh for running executions
  useEffect(() => {
    if (!autoRefresh || !isRunning || !benchmarkId || !executionId) return;

    const interval = setInterval(loadTransactions, 2000);
    return () => clearInterval(interval);
  }, [autoRefresh, isRunning, benchmarkId, executionId, loadTransactions]);

  // Load on mount and when execution changes
  useEffect(() => {
    loadTransactions();
  }, [loadTransactions]);

  const toggleExpanded = useCallback((id: string) => {
    setExpandedItems(prev => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  }, []);

  const toggleExpandAll = useCallback(() => {
    if (expandedItems.size === transactions.length) {
      setExpandedItems(new Set());
    } else {
      setExpandedItems(new Set(transactions.map(t => t.id)));
    }
  }, [expandedItems.size, transactions]);

  const clearLogs = useCallback(() => {
    setTransactions([]);
    setError(null);
  }, []);

  const exportLogs = useCallback(() => {
    const data = JSON.stringify(transactions, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `transactions-${benchmarkId}-${executionId}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [transactions, benchmarkId, executionId]);

  const filteredTransactions = transactions.filter(tx =>
    !filter ||
    tx.type.toLowerCase().includes(filter.toLowerCase()) ||
    tx.status.toLowerCase().includes(filter.toLowerCase()) ||
    JSON.stringify(tx.data).toLowerCase().includes(filter.toLowerCase())
  );

  const getStatusColor = (status: string) => {
    switch (status) {
      case "success": return "text-green-600 bg-green-50 border-green-200";
      case "failed": return "text-red-600 bg-red-50 border-red-200";
      case "pending": return "text-yellow-600 bg-yellow-50 border-yellow-200";
      default: return "text-gray-600 bg-gray-50 border-gray-200";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "success": return "✓";
      case "failed": return "✗";
      case "pending": return "⏳";
      default: return "•";
    }
  };

  if (!benchmarkId || !executionId) {
    return (
      <div className="p-4 bg-white border rounded-lg">
        <h3 className="text-lg font-semibold mb-3">Transaction Log</h3>
        <div className="text-gray-500 text-center py-8">
          <svg class="w-12 h-12 mx-auto mb-3 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
          </svg>
          <p>Select a benchmark execution to view transaction logs</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 bg-white border rounded-lg">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Transaction Log</h3>
        <div className="flex items-center space-x-2">
          {isRunning && (
            <div className="flex items-center text-xs text-green-600">
              <div className="w-2 h-2 bg-green-500 rounded-full mr-1 animate-pulse"></div>
              Live
            </div>
          )}
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`px-2 py-1 text-xs rounded ${
              autoRefresh
                ? "bg-green-100 text-green-700 border border-green-200"
                : "bg-gray-100 text-gray-700 border border-gray-200"
            }`}
          >
            Auto-refresh
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center space-x-2 mb-4">
        <input
          type="text"
          value={filter}
          onChange={(e) => setFilter(e.currentTarget.value)}
          placeholder="Filter transactions..."
          className="flex-1 px-3 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        <button
          onClick={toggleExpandAll}
          className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
        >
          {expandedItems.size === transactions.length ? "Collapse All" : "Expand All"}
        </button>
        <button
          onClick={clearLogs}
          className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors"
        >
          Clear
        </button>
        <button
          onClick={exportLogs}
          disabled={transactions.length === 0}
          className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Export
        </button>
      </div>

      {/* Transaction List */}
      <div className="border rounded-lg overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500 mr-2"></div>
            <span className="text-sm text-gray-600">Loading transactions...</span>
          </div>
        ) : error ? (
          <div className="p-4 text-center">
            <div className="text-red-500 mb-2">
              <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <p className="text-sm text-red-600 mb-2">{error}</p>
            <button
              onClick={loadTransactions}
              className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
            >
              Retry
            </button>
          </div>
        ) : filteredTransactions.length === 0 ? (
          <div className="p-4 text-center text-gray-500">
            {filter ? "No transactions match the filter" : "No transactions recorded yet"}
          </div>
        ) : (
          <div className="divide-y">
            {filteredTransactions.map((transaction) => (
              <div key={transaction.id} className="border-l-4 border-transparent hover:border-gray-200 transition-colors">
                <div
                  className="p-3 cursor-pointer hover:bg-gray-50 transition-colors"
                  onClick={() => toggleExpanded(transaction.id)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <span className={`px-2 py-1 text-xs font-medium rounded border ${getStatusColor(transaction.status)}`}>
                        {getStatusIcon(transaction.status)} {transaction.status}
                      </span>
                      <span className="text-sm font-medium text-gray-900">
                        {transaction.type}
                      </span>
                      <span className="text-xs text-gray-500">
                        {new Date(transaction.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className="text-xs text-gray-400">
                        ID: {transaction.id}
                      </span>
                      <svg
                        className={`w-4 h-4 text-gray-400 transition-transform ${
                          expandedItems.has(transaction.id) ? "rotate-90" : ""
                        }`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                      </svg>
                    </div>
                  </div>

                  {transaction.error && (
                    <div className="mt-2 text-sm text-red-600 bg-red-50 p-2 rounded border border-red-200">
                      {transaction.error}
                    </div>
                  )}
                </div>

                {/* Expanded Details */}
                {expandedItems.has(transaction.id) && (
                  <div className="px-3 pb-3 bg-gray-50 border-t">
                    <div className="pt-3">
                      <div className="text-xs font-medium text-gray-700 mb-2">Transaction Details:</div>
                      <pre className="text-xs bg-white p-2 rounded border overflow-x-auto">
                        {JSON.stringify(transaction.data, null, 2)}
                      </pre>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer Info */}
      <div className="mt-3 text-xs text-gray-500 flex items-center justify-between">
        <span>
          {filteredTransactions.length} transaction{filteredTransactions.length !== 1 ? "s" : ""}
          {filter && ` (filtered from ${transactions.length})`}
        </span>
        <span>
          Benchmark: {benchmarkId} | Execution: {executionId}
        </span>
      </div>
    </div>
  );
}
