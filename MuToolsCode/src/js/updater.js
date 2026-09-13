// 更新检测模块
import { showToast } from "./toast.js";
import { addLog } from "./logger.js";

const { invoke } = window.__TAURI__.core;

export const DEFAULT_UPDATE_URL = "https://mutools.netlify.app/update_info.json";

export function getUpdateUrl() {
  try {
    const saved = JSON.parse(localStorage.getItem("mutools_settings") || "{}");
    return saved.updateUrl || DEFAULT_UPDATE_URL;
  } catch {
    return DEFAULT_UPDATE_URL;
  }
}

// 打开外部链接
export function openExternalUrl(url) {
  if (!url) return;
  if (window.__TAURI__ && window.__TAURI__.opener) {
    window.__TAURI__.opener.openUrl(url).catch(err => {
      console.error("无法打开链接:", err);
    });
  } else {
    window.open(url, "_blank");
  }
}

// 显示发现新版本的提示
function showUpdateAvailable(data) {
  if (!document.getElementById("modal-update-overlay")) return;

  const title = document.getElementById("update-modal-title");
  const version = document.getElementById("update-modal-version");
  const date = document.getElementById("update-modal-date");
  const info = document.getElementById("update-modal-info");

  if (title) title.textContent = "发现新版本";
  if (version) version.textContent = (typeof data.version === "string" ? data.version : "") || "未知版本";
  if (date) date.textContent = (typeof data.date === "string" ? data.date : "") || "";
  if (info) info.innerHTML = (typeof data.info === "string" ? data.info : "") || "";

  document.getElementById("modal-update-overlay").style.display = "flex";
}

// 渲染更新地址选择列表
function renderUpdateLinks(list) {
  const container = document.getElementById("update-modal-links");
  if (!container) return;
  container.innerHTML = "";

  if (!list || list.length === 0) {
    container.innerHTML = '<div class="update-links-empty">暂无更新地址</div>';
    return;
  }

  list.forEach((item, index) => {
    const btn = document.createElement("button");
    btn.className = "btn-secondary update-link-btn";
    btn.textContent = item.name || `地址 ${index + 1}`;
    btn.addEventListener("click", () => openExternalUrl(item.url));
    container.appendChild(btn);
  });
}

// 执行更新检测
export async function checkForUpdate(manual = false) {
  const url = getUpdateUrl();
  addLog(`检测更新: ${url}`);

  let data;
  try {
    // 通过后端请求，绕过前端 CORS 限制
    const text = await invoke("fetch_update_info", { url });
    data = JSON.parse(text);
  } catch (e) {
    console.error("更新检测请求失败:", e);
    if (manual) {
      showToast("检测更新失败：" + e, "error");
      addLog(`检测更新失败: ${e}`);
    }
    return;
  }

  if (!data || typeof data !== "object") {
    if (manual) showToast("更新信息格式错误", "error");
    return;
  }

  const latest = data.version;
  const current = (await invoke("get_app_version").catch(() => null)) || "1.2.0";

  addLog(`当前版本: ${current}，最新版本: ${latest}`);

  try {
    const hasUpdate = compareVersions(latest, current) > 0;
    if (hasUpdate) {
      showUpdateAvailable(data);
      renderUpdateLinks(data.updateUrl || data.urls || data.links);
    } else if (manual) {
      showToast("当前已是最新版本", "success");
    }
  } catch (e) {
    console.error("版本比较失败:", e);
    if (manual) showToast("已是最新版本或无法比较版本", "info");
  }
}

// 简单版本号比较（支持 x.y.z 格式）
function compareVersions(a, b) {
  const pa = String(a || "").split(".").map(n => parseInt(n, 10) || 0);
  const pb = String(b || "").split(".").map(n => parseInt(n, 10) || 0);
  const len = Math.max(pa.length, pb.length);
  for (let i = 0; i < len; i++) {
    const x = pa[i] || 0;
    const y = pb[i] || 0;
    if (x > y) return 1;
    if (x < y) return -1;
  }
  return 0;
}

// 初始化更新模态框事件
export function initUpdaterPage() {
  const overlay = document.getElementById("modal-update-overlay");
  if (!overlay) return;

  const close = () => { overlay.style.display = "none"; };

  const btnClose = document.getElementById("btn-update-close");
  const btnCloseBottom = document.getElementById("btn-update-close-bottom");
  if (btnClose) btnClose.addEventListener("click", close);
  if (btnCloseBottom) btnCloseBottom.addEventListener("click", close);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });
}