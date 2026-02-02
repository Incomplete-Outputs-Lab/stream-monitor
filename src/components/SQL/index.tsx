import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import CodeMirror from "@uiw/react-codemirror";
import { sql } from "@codemirror/lang-sql";
import type {
  SqlQueryResult,
  SqlTemplate,
  SaveTemplateRequest,
  TableInfo,
} from "../../types";
import { toast } from "../../utils/toast";
import { confirm } from "../../utils/confirm";

export function SQLViewer() {
  const [query, setQuery] = useState<string>(
    "-- Select all channels\nSELECT * FROM channels LIMIT 100;"
  );
  const [result, setResult] = useState<SqlQueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);

  const [templates, setTemplates] = useState<SqlTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<SqlTemplate | null>(
    null
  );
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [templateName, setTemplateName] = useState("");
  const [templateDescription, setTemplateDescription] = useState("");

  const [tables, setTables] = useState<TableInfo[]>([]);
  const [showTables, setShowTables] = useState(true);
  const [dbInfo, setDbInfo] = useState<{ path: string; size_bytes: number } | null>(null);

  // テンプレート一覧を読み込み
  const loadTemplates = async () => {
    try {
      const result = await invoke<SqlTemplate[]>("list_sql_templates");
      setTemplates(result);
    } catch (err) {
      console.error("Failed to load templates:", err);
    }
  };

  // テーブル一覧を読み込み
  const loadTables = async () => {
    try {
      const result = await invoke<TableInfo[]>("list_database_tables");
      setTables(result);
    } catch (err) {
      console.error("Failed to load tables:", err);
    }
  };

  // データベース情報を読み込み
  const loadDbInfo = async () => {
    try {
      const result = await invoke<{ path: string; size_bytes: number }>(
        "get_database_info"
      );
      setDbInfo(result);
    } catch (err) {
      console.error("Failed to load database info:", err);
    }
  };

  useEffect(() => {
    loadTemplates();
    loadTables();
    loadDbInfo();
  }, []);

  // SQLクエリを実行
  const executeQuery = async () => {
    if (!query.trim()) {
      setError("クエリが空です");
      return;
    }

    setIsExecuting(true);
    setError(null);
    setResult(null);

    try {
      const result = await invoke<SqlQueryResult>("execute_sql", { query });
      setResult(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsExecuting(false);
    }
  };

  // テンプレートを保存
  const saveTemplate = async () => {
    if (!templateName.trim()) {
      toast.warning("テンプレート名を入力してください");
      return;
    }

    try {
      const request: SaveTemplateRequest = {
        id: selectedTemplate?.id,
        name: templateName,
        description: templateDescription || undefined,
        query,
      };
      await invoke("save_sql_template", { request });
      setShowSaveDialog(false);
      setTemplateName("");
      setTemplateDescription("");
      setSelectedTemplate(null);
      await loadTemplates();
    } catch (err) {
      toast.error(`テンプレートの保存に失敗しました: ${err}`);
    }
  };

  // テンプレートを削除
  const deleteTemplate = async (id: number) => {
    const confirmed = await confirm({
      title: 'テンプレートの削除',
      message: 'このテンプレートを削除しますか？',
      confirmText: '削除',
      type: 'danger',
    });
    
    if (!confirmed) {
      return;
    }

    try {
      await invoke("delete_sql_template", { id });
      await loadTemplates();
      if (selectedTemplate?.id === id) {
        setSelectedTemplate(null);
      }
    } catch (err) {
      toast.error(`テンプレートの削除に失敗しました: ${err}`);
    }
  };

  // テンプレートを読み込み
  const loadTemplate = (template: SqlTemplate) => {
    setQuery(template.query);
    setSelectedTemplate(template);
    setResult(null);
    setError(null);
  };

  // テーブル名をクエリに挿入
  const insertTableName = (tableName: string) => {
    setQuery((prev) => {
      if (prev.trim() === "" || prev.trim() === "-- Select all channels\nSELECT * FROM channels LIMIT 100;") {
        return `SELECT * FROM ${tableName} LIMIT 100;`;
      }
      return prev + ` ${tableName}`;
    });
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          SQLビューワー
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          DuckDBデータベースにSQLクエリを実行
        </p>
        {dbInfo && (
          <div className="mt-2 text-xs text-gray-500 dark:text-gray-400 font-mono select-text">
            📁 {dbInfo.path} ({(dbInfo.size_bytes / 1024 / 1024).toFixed(2)} MB)
          </div>
        )}
      </div>

      <div className="grid grid-cols-12 gap-6">
        {/* サイドバー: テーブル一覧とテンプレート一覧 */}
        <div className="col-span-3 space-y-4">
          {/* テーブル一覧 */}
          <div className="card p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                テーブル
              </h3>
              <button
                onClick={() => setShowTables(!showTables)}
                className="p-1 hover:bg-gray-100 dark:hover:bg-slate-700 rounded transition-colors"
                title={showTables ? "非表示" : "表示"}
              >
                <svg
                  className={`w-4 h-4 text-gray-600 dark:text-gray-400 transition-transform ${
                    showTables ? "" : "rotate-180"
                  }`}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 15l7-7 7 7"
                  />
                </svg>
              </button>
            </div>

            {showTables && (
              <div className="space-y-1 max-h-64 overflow-y-auto">
                {tables.length === 0 ? (
                  <p className="text-xs text-gray-500 dark:text-gray-400 text-center py-4">
                    テーブルがありません
                  </p>
                ) : (
                  tables.map((table) => (
                    <button
                      key={table.table_name}
                      onClick={() => insertTableName(table.table_name)}
                      className="w-full text-left p-2 rounded hover:bg-indigo-50 dark:hover:bg-indigo-900/20 transition-colors group"
                      title={`クリックしてクエリに挿入 (${table.column_count}列)`}
                    >
                      <div className="flex items-center gap-2">
                        <svg
                          className="w-4 h-4 text-indigo-500 dark:text-indigo-400 flex-shrink-0"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
                          />
                        </svg>
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                            {table.table_name}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {table.column_count}列
                          </p>
                        </div>
                      </div>
                    </button>
                  ))
                )}
              </div>
            )}
          </div>

          {/* テンプレート一覧 */}
          <div className="card p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                テンプレート
              </h3>
              <button
                onClick={() => {
                  setSelectedTemplate(null);
                  setTemplateName("");
                  setTemplateDescription("");
                  setShowSaveDialog(true);
                }}
                className="p-1 hover:bg-gray-100 dark:hover:bg-slate-700 rounded transition-colors"
                title="新規テンプレート作成"
              >
                <svg
                  className="w-4 h-4 text-gray-600 dark:text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 4v16m8-8H4"
                  />
                </svg>
              </button>
            </div>

            <div className="space-y-1 max-h-64 overflow-y-auto">
              {templates.length === 0 ? (
                <p className="text-xs text-gray-500 dark:text-gray-400 text-center py-4">
                  保存されたテンプレートはありません
                </p>
              ) : (
                templates.map((template) => (
                  <div
                    key={template.id}
                    className={`group relative p-2 rounded cursor-pointer transition-colors ${
                      selectedTemplate?.id === template.id
                        ? "bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800"
                        : "hover:bg-gray-50 dark:hover:bg-slate-700/50"
                    }`}
                    onClick={() => loadTemplate(template)}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                          {template.name}
                        </p>
                        {template.description && (
                          <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                            {template.description}
                          </p>
                        )}
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteTemplate(template.id!);
                        }}
                        className="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-100 dark:hover:bg-red-900/30 rounded transition-all"
                        title="削除"
                      >
                        <svg
                          className="w-3 h-3 text-red-600 dark:text-red-400"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                          />
                        </svg>
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        {/* メインエリア: エディタと結果 */}
        <div className="col-span-9 space-y-4">
          {/* SQLエディタ */}
          <div className="card p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                SQLクエリ
              </h3>
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    setShowSaveDialog(true);
                    if (selectedTemplate) {
                      setTemplateName(selectedTemplate.name);
                      setTemplateDescription(
                        selectedTemplate.description || ""
                      );
                    }
                  }}
                  className="px-3 py-1.5 text-sm bg-gray-100 hover:bg-gray-200 dark:bg-slate-700 dark:hover:bg-slate-600 text-gray-700 dark:text-gray-300 rounded transition-colors"
                >
                  💾 保存
                </button>
                <button
                  onClick={executeQuery}
                  disabled={isExecuting}
                  className="px-4 py-1.5 text-sm bg-gradient-to-r from-purple-500 to-indigo-600 hover:from-purple-600 hover:to-indigo-700 text-white rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-md hover:shadow-lg"
                >
                  {isExecuting ? "実行中..." : "▶ 実行"}
                </button>
              </div>
            </div>

            <div className="border border-gray-200 dark:border-slate-700 rounded overflow-hidden">
              <CodeMirror
                value={query}
                height="200px"
                extensions={[sql()]}
                onChange={(value) => setQuery(value)}
                theme="dark"
                className="text-sm"
              />
            </div>
          </div>

          {/* エラー表示 */}
          {error && (
            <div className="card p-4 bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800">
              <div className="flex gap-3">
                <svg
                  className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <div>
                  <h4 className="text-sm font-semibold text-red-900 dark:text-red-200">
                    エラー
                  </h4>
                  <pre className="text-xs text-red-700 dark:text-red-300 mt-1 whitespace-pre-wrap font-mono select-text">
                    {error}
                  </pre>
                </div>
              </div>
            </div>
          )}

          {/* クエリ結果 */}
          {result && (
            <div className="card p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  実行結果
                </h3>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  実行時間: {result.execution_time_ms}ms
                  {result.affected_rows !== undefined &&
                    ` | 影響行数: ${result.affected_rows}`}
                  {result.rows.length > 0 && ` | 行数: ${result.rows.length}`}
                </div>
              </div>

              {result.rows.length > 0 ? (
                <div className="overflow-auto max-h-96 border border-gray-200 dark:border-slate-700 rounded">
                  <table className="w-full text-sm">
                    <thead className="bg-gray-50 dark:bg-slate-800 sticky top-0">
                      <tr>
                        {result.columns.map((col, i) => (
                          <th
                            key={`col-${col}-${i}`}
                            className="px-3 py-2 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-slate-700"
                          >
                            {col}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="bg-white dark:bg-slate-900">
                      {result.rows.map((row, i) => (
                        <tr
                          key={`row-${i}`}
                          className="border-b border-gray-100 dark:border-slate-800 hover:bg-gray-50 dark:hover:bg-slate-800/50"
                        >
                          {row.map((cell, j) => {
                            // 配列かどうかを判定
                            const isArray = Array.isArray(cell);
                            
                            return (
                              <td
                                key={`cell-${i}-${j}`}
                                className="px-3 py-2 text-gray-900 dark:text-gray-100 font-mono text-xs select-text"
                              >
                                {cell === null ? (
                                  <span className="text-gray-400 dark:text-gray-600 italic">
                                    NULL
                                  </span>
                                ) : isArray ? (
                                  // 配列の場合は見やすく表示（カンマ区切りまたはバッジ形式）
                                  <div className="flex flex-col gap-1">
                                    {(cell as unknown[]).length === 0 ? (
                                      <span className="text-gray-400 dark:text-gray-600 italic text-xs">
                                        []
                                      </span>
                                    ) : (
                                      <>
                                        {/* カンマ区切り表示 */}
                                        <div className="text-xs text-gray-700 dark:text-gray-300">
                                          {(cell as unknown[]).map(String).join(", ")}
                                        </div>
                                        {/* バッジ形式表示（要素が10個以下の場合のみ） */}
                                        {(cell as unknown[]).length <= 10 && (
                                          <div className="flex flex-wrap gap-1">
                                            {(cell as unknown[]).map((item, idx) => (
                                              <span
                                                key={idx}
                                                className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-100 dark:bg-indigo-900/30 text-indigo-800 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800"
                                              >
                                                {String(item)}
                                              </span>
                                            ))}
                                          </div>
                                        )}
                                      </>
                                    )}
                                  </div>
                                ) : typeof cell === "object" ? (
                                  // その他のオブジェクトはJSON表示
                                  <span className="text-xs">
                                    {JSON.stringify(cell)}
                                  </span>
                                ) : (
                                  String(cell)
                                )}
                              </td>
                            );
                          })}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : result.affected_rows !== undefined ? (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  <svg
                    className="w-12 h-12 mx-auto mb-3 text-green-500"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  <p className="text-sm">
                    クエリが正常に実行されました
                    {result.affected_rows > 0 &&
                      ` (${result.affected_rows}行に影響)`}
                  </p>
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  結果がありません
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* テンプレート保存ダイアログ */}
      {showSaveDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-slate-800 rounded-lg shadow-xl p-6 w-full max-w-md">
            {/* タイトルとアイコン */}
            <div className="flex items-center gap-3 mb-4">
              {selectedTemplate ? (
                <div className="flex-shrink-0 w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
                  <svg
                    className="w-6 h-6 text-blue-600 dark:text-blue-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                    />
                  </svg>
                </div>
              ) : (
                <div className="flex-shrink-0 w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
                  <svg
                    className="w-6 h-6 text-green-600 dark:text-green-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 4v16m8-8H4"
                    />
                  </svg>
                </div>
              )}
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {selectedTemplate ? "テンプレートを編集" : "新規テンプレート作成"}
              </h3>
            </div>

            {/* 編集中のテンプレート情報 */}
            {selectedTemplate && (
              <div className="mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                <div className="flex items-start gap-2">
                  <svg
                    className="w-4 h-4 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium text-blue-900 dark:text-blue-200">
                      編集中: {selectedTemplate.name}
                    </p>
                    {selectedTemplate.description && (
                      <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
                        {selectedTemplate.description}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  テンプレート名 <span className="text-red-500">*</span>
                </label>
                <input
                  type="text"
                  value={templateName}
                  onChange={(e) => setTemplateName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  placeholder="例: 全チャンネル取得"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  説明（任意）
                </label>
                <textarea
                  value={templateDescription}
                  onChange={(e) => setTemplateDescription(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  rows={3}
                  placeholder="このクエリの説明を入力..."
                />
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={() => {
                  setShowSaveDialog(false);
                  setTemplateName("");
                  setTemplateDescription("");
                }}
                className="flex-1 px-4 py-2 border border-gray-300 dark:border-slate-600 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-50 dark:hover:bg-slate-700 transition-colors"
              >
                キャンセル
              </button>
              <button
                onClick={saveTemplate}
                className={`flex-1 px-4 py-2 text-white rounded-md transition-all shadow-md hover:shadow-lg ${
                  selectedTemplate
                    ? "bg-gradient-to-r from-blue-500 to-indigo-600 hover:from-blue-600 hover:to-indigo-700"
                    : "bg-gradient-to-r from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700"
                }`}
              >
                {selectedTemplate ? "更新" : "新規保存"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
