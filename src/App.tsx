import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ChevronLeft,
  ChevronRight,
  RotateCw,
  Pin,
  PinOff,
  Eye,
  EyeOff,
  Sparkles,
  Minus,
  X,
  Tv,
  Music,
  Video,
  Globe,
  Maximize2,
  ChevronUp,
  ChevronDown,
  ExternalLink
} from "lucide-react";
import "./App.css";

interface Service {
  id: string;
  name: string;
  url: string;
  icon: React.ReactNode;
}

const SERVICES: Service[] = [
  {
    id: "prime_video",
    name: "Prime Video",
    url: "https://www.amazon.co.jp/gp/video/storefront",
    icon: <Tv className="w-4 h-4 text-sky-400" />
  },
  {
    id: "youtube_music",
    name: "YouTube Music",
    url: "https://music.youtube.com",
    icon: <Music className="w-4 h-4 text-red-400" />
  },
  {
    id: "youtube",
    name: "YouTube",
    url: "https://www.youtube.com",
    icon: <Video className="w-4 h-4 text-red-500" />
  },
  {
    id: "twitch",
    name: "Twitch",
    url: "https://www.twitch.tv",
    icon: <Globe className="w-4 h-4 text-purple-400" />
  }
];

export default function App() {
  const [selectedService, setSelectedService] = useState<string>("prime_video");
  const [customUrl, setCustomUrl] = useState<string>("");
  const [showCustomModal, setShowCustomModal] = useState<boolean>(false);
  const [alwaysOnTop, setAlwaysOnTop] = useState<boolean>(true);
  const [clickThrough, setClickThrough] = useState<boolean>(false);
  const [barVisible, setBarVisible] = useState<boolean>(true);
  const [showToast, setShowToast] = useState<string | null>(null);

  const isDraggingRef = React.useRef(false);
  const lastMousePosRef = React.useRef({ x: 0, y: 0 });

  // ドラッグ移動ハンドラ (マウス追従)
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;
      const deltaX = e.screenX - lastMousePosRef.current.x;
      const deltaY = e.screenY - lastMousePosRef.current.y;
      lastMousePosRef.current = { x: e.screenX, y: e.screenY };

      if (deltaX !== 0 || deltaY !== 0) {
        invoke("move_window_by", { deltaX, deltaY }).catch(console.error);
      }
    };

    const handleMouseUp = () => {
      isDraggingRef.current = false;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  const handleDragStart = (e: React.MouseEvent) => {
    if (e.button === 0) {
      isDraggingRef.current = true;
      lastMousePosRef.current = { x: e.screenX, y: e.screenY };
      invoke("start_drag").catch(() => {});
    }
  };

  useEffect(() => {
    const unlistenClickThrough = listen<boolean>("click-through-changed", (event) => {
      setClickThrough(event.payload);
      triggerToast(event.payload ? "クリックスルー: ON (Cmd+Shift+Xで解除)" : "クリックスルー: OFF");
    });

    const unlistenBarVisibility = listen<boolean>("bar-visibility-changed", (event) => {
      setBarVisible(event.payload);
    });

    // 初期化時にサイズ同期
    invoke("sync_bounds").catch(console.error);

    return () => {
      unlistenClickThrough.then((fn) => fn());
      unlistenBarVisibility.then((fn) => fn());
    };
  }, []);

  const triggerToast = (msg: string) => {
    setShowToast(msg);
    setTimeout(() => setShowToast(null), 3000);
  };

  const handleSelectService = async (service: Service) => {
    setSelectedService(service.id);
    try {
      await invoke("set_url", { url: service.url });
      triggerToast(`${service.name} に切り替えました`);
    } catch (e) {
      console.error(e);
    }
  };

  const handleCustomUrlSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customUrl) return;
    let url = customUrl;
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      url = "https://" + url;
    }
    try {
      await invoke("set_url", { url });
      setSelectedService("custom");
      setShowCustomModal(false);
      triggerToast("カスタムURLを読み込みました");
    } catch (e) {
      console.error(e);
    }
  };

  const toggleAlwaysOnTop = async () => {
    const next = !alwaysOnTop;
    setAlwaysOnTop(next);
    await invoke("set_always_on_top", { enabled: next });
    triggerToast(next ? "最前面固定: ON" : "最前面固定: OFF");
  };

  const toggleClickThroughMode = async () => {
    try {
      const next = await invoke<boolean>("toggle_click_through");
      setClickThrough(next);
    } catch (e) {
      console.error(e);
    }
  };

  const handleOptimizePrimeVideo = async () => {
    try {
      await invoke("optimize_prime_video");
      triggerToast("Prime Video UIを最適化しました");
    } catch (e) {
      console.error(e);
    }
  };

  const handleSetPresetSize = async (w: number, h: number) => {
    try {
      await invoke("set_preset_size", { width: w, height: h });
      triggerToast(`サイズ変更: ${w}x${h}`);
    } catch (e) {
      console.error(e);
    }
  };

  const toggleBar = async () => {
    try {
      const next = await invoke<boolean>("toggle_bar_visible");
      setBarVisible(next);
    } catch (e) {
      console.error(e);
    }
  };

  if (!barVisible) {
    return (
      <div
        onClick={toggleBar}
        className="w-full h-full bg-black/60 hover:bg-sky-600/80 cursor-pointer flex items-center justify-center transition-all group"
        title="コントロールバーを展開"
      >
        <ChevronDown className="w-3 h-3 text-white/70 group-hover:text-white" />
      </div>
    );
  }

  return (
    <div className="w-full h-11 bg-zinc-900/95 backdrop-blur-md border-b border-zinc-800 text-zinc-200 flex items-center justify-between px-3 select-none text-xs">
      {/* 左側: サービスセレクター & ブラウザ操作 */}
      <div className="flex items-center gap-1.5 no-drag">
        {/* サービスドロップダウン */}
        <div className="relative group">
          <button className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 font-medium text-white transition">
            {SERVICES.find((s) => s.id === selectedService)?.icon || <Globe className="w-4 h-4 text-emerald-400" />}
            <span className="max-w-[85px] truncate">
              {selectedService === "custom" ? "Custom" : SERVICES.find((s) => s.id === selectedService)?.name || "Select"}
            </span>
            <ChevronDown className="w-3 h-3 text-zinc-400" />
          </button>
          
          <div className="absolute left-0 top-full mt-1 w-44 bg-zinc-800 border border-zinc-700 rounded-md shadow-xl py-1 hidden group-hover:block z-50">
            {SERVICES.map((s) => (
              <button
                key={s.id}
                onClick={() => handleSelectService(s)}
                className={`w-full flex items-center gap-2 px-3 py-1.5 text-left hover:bg-zinc-700 transition ${
                  selectedService === s.id ? "bg-zinc-700/60 font-semibold text-white" : "text-zinc-300"
                }`}
              >
                {s.icon}
                <span>{s.name}</span>
              </button>
            ))}
            <div className="border-t border-zinc-700 my-1"></div>
            <button
              onClick={() => setShowCustomModal(true)}
              className="w-full flex items-center gap-2 px-3 py-1.5 text-left hover:bg-zinc-700 text-zinc-300 transition"
            >
              <ExternalLink className="w-4 h-4 text-emerald-400" />
              <span>URL指定...</span>
            </button>
          </div>
        </div>

        {/* ナビゲーションボタン */}
        <div className="flex items-center bg-zinc-800/80 rounded p-0.5">
          <button
            onClick={() => invoke("nav_back")}
            className="p-1 hover:bg-zinc-700 rounded text-zinc-300 hover:text-white"
            title="戻る"
          >
            <ChevronLeft className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => invoke("nav_forward")}
            className="p-1 hover:bg-zinc-700 rounded text-zinc-300 hover:text-white"
            title="進む"
          >
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => invoke("nav_reload")}
            className="p-1 hover:bg-zinc-700 rounded text-zinc-300 hover:text-white"
            title="再読み込み"
          >
            <RotateCw className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Prime Video 最適化ボタン */}
        {selectedService === "prime_video" && (
          <button
            onClick={handleOptimizePrimeVideo}
            className="flex items-center gap-1 px-2 py-1 bg-sky-950/60 text-sky-300 hover:bg-sky-900 border border-sky-800/60 rounded transition"
            title="Prime Video のヘッダーを縮小して動画表示を最適化"
          >
            <Sparkles className="w-3 h-3" />
            <span className="text-[10px]">最適化</span>
          </button>
        )}
      </div>

      {/* 中央: ウィンドウドラッグ領域 */}
      <div
        data-tauri-drag-region
        onMouseDown={handleDragStart}
        className="flex-1 h-full flex items-center justify-center cursor-grab active:cursor-grabbing px-2 select-none"
      >
        <div data-tauri-drag-region className="flex items-center gap-1 text-zinc-500 hover:text-zinc-300 transition text-[11px] pointer-events-none">
          <span>:::</span>
          <span className="font-mono text-[10px] text-zinc-400">Overlay (Drag to Move)</span>
          <span>:::</span>
        </div>
      </div>


      {/* 右側: ウィンドウ制御 & オーバーレイ設定 */}
      <div className="flex items-center gap-1.5 no-drag">
        {/* 最前面固定 */}
        <button
          onClick={toggleAlwaysOnTop}
          className={`p-1.5 rounded transition ${
            alwaysOnTop ? "bg-amber-500/20 text-amber-300 hover:bg-amber-500/30" : "bg-zinc-800 text-zinc-400 hover:text-zinc-200"
          }`}
          title={alwaysOnTop ? "最前面固定中 (クリックで解除)" : "最前面固定"}
        >
          {alwaysOnTop ? <Pin className="w-3.5 h-3.5" /> : <PinOff className="w-3.5 h-3.5" />}
        </button>

        {/* クリックスルー (操作透過) */}
        <button
          onClick={toggleClickThroughMode}
          className={`p-1.5 rounded flex items-center gap-1 transition ${
            clickThrough ? "bg-indigo-600 text-white" : "bg-zinc-800 text-zinc-400 hover:text-zinc-200"
          }`}
          title="クリックスルー (Cmd+Shift+X で解除)"
        >
          {clickThrough ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
        </button>

        {/* サイズプリセットメニュー */}
        <div className="relative group">
          <button
            className="p-1.5 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-400 hover:text-zinc-200 transition"
            title="サイズプリセット"
          >
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
          <div className="absolute right-0 top-full mt-1 w-32 bg-zinc-800 border border-zinc-700 rounded-md shadow-xl py-1 hidden group-hover:block z-50">
            <button
              onClick={() => handleSetPresetSize(480, 270)}
              className="w-full px-3 py-1 text-left text-zinc-300 hover:bg-zinc-700 text-[11px]"
            >
              小 (480x270)
            </button>
            <button
              onClick={() => handleSetPresetSize(720, 405)}
              className="w-full px-3 py-1 text-left text-zinc-300 hover:bg-zinc-700 text-[11px]"
            >
              中 (720x405)
            </button>
            <button
              onClick={() => handleSetPresetSize(960, 540)}
              className="w-full px-3 py-1 text-left text-zinc-300 hover:bg-zinc-700 text-[11px]"
            >
              大 (960x540)
            </button>
          </div>
        </div>

        {/* バーの折りたたみ */}
        <button
          onClick={toggleBar}
          className="p-1.5 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-400 hover:text-zinc-200 transition"
          title="コントロールバーを隠す"
        >
          <ChevronUp className="w-3.5 h-3.5" />
        </button>

        <div className="w-[1px] h-4 bg-zinc-700 mx-0.5"></div>

        {/* 最小化 & 閉じる */}
        <button
          onClick={() => invoke("minimize_app")}
          className="p-1.5 hover:bg-zinc-700 rounded text-zinc-400 hover:text-zinc-200"
          title="最小化"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => invoke("close_app")}
          className="p-1.5 hover:bg-rose-600 rounded text-zinc-400 hover:text-white transition"
          title="閉じる"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* トースト通知 */}
      {showToast && (
        <div className="absolute top-12 left-1/2 -translate-x-1/2 bg-zinc-950/90 text-white px-3 py-1.5 rounded-full shadow-lg border border-zinc-700 text-[11px] pointer-events-none animate-fade-in z-50 flex items-center gap-1.5">
          <span>{showToast}</span>
        </div>
      )}

      {/* カスタムURLモーダル */}
      {showCustomModal && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50 no-drag">
          <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-4 w-80 shadow-2xl text-left">
            <h3 className="text-sm font-semibold text-white mb-2">カスタムURLを開く</h3>
            <form onSubmit={handleCustomUrlSubmit}>
              <input
                type="text"
                value={customUrl}
                onChange={(e) => setCustomUrl(e.target.value)}
                placeholder="https://..."
                className="w-full px-3 py-1.5 bg-zinc-800 border border-zinc-700 rounded text-white text-xs mb-3 focus:outline-none focus:border-sky-500"
                autoFocus
              />
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowCustomModal(false)}
                  className="px-3 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded text-xs"
                >
                  キャンセル
                </button>
                <button
                  type="submit"
                  className="px-3 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded text-xs font-medium"
                >
                  読み込む
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

