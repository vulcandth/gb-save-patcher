import { SITE_CONFIG } from "./config.js";

const ui = {
  advancedBox: document.getElementById("advancedBox"),
  inputFile: document.getElementById("inputFile"),
  dropZone: document.getElementById("dropZone"),
  dropZoneHint: document.getElementById("dropZoneHint"),
  targetVersion: document.getElementById("targetVersion"),
  devType: document.getElementById("devType"),
  patchBtn: document.getElementById("patchBtn"),
  downloadLink: document.getElementById("downloadLink"),
  downloadLogBtn: document.getElementById("downloadLogBtn"),
  clearLogBtn: document.getElementById("clearLogBtn"),
  status: document.getElementById("status"),
  wasmState: document.getElementById("wasmState"),
  inputMeta: document.getElementById("inputMeta"),
  detectedVersion: document.getElementById("detectedVersion"),
  outputMeta: document.getElementById("outputMeta"),
  logOutput: document.getElementById("logOutput"),
  instructionsList: document.getElementById("instructionsList"),
  gameTitle: document.getElementById("gameTitle"),
  gameSubtitle: document.getElementById("gameSubtitle"),
  githubLink: document.getElementById("githubLink"),
  githubLinkLabel: document.getElementById("githubLinkLabel"),
};

let wasm = null;

const state = {
  bytes: null,
  fileName: null,
  detectedVersion: null,
  downloadUrl: null,
  logMessages: [],
  lastTargetLog: null,
};

function nowTime() {
  return new Date().toLocaleTimeString();
}

function updateLogButtons() {
  ui.downloadLogBtn.disabled = state.logMessages.length === 0;
}

function clearLog() {
  state.logMessages = [];
  ui.logOutput.textContent = "";
  updateLogButtons();
}

function logMessage(message, level = "info") {
  const prefix = level === "error" ? "ERROR: " : level === "warn" ? "WARN: " : "";
  const line = `[${nowTime()}] ${prefix}${message}`;
  state.logMessages.push({ line, level });

  if (ui.logOutput.textContent.length > 0) {
    ui.logOutput.textContent += "\n";
  }
  ui.logOutput.textContent += line;
  ui.logOutput.scrollTop = ui.logOutput.scrollHeight;
  updateLogButtons();
}

function downloadLog() {
  const content = state.logMessages.map((m) => m.line).join("\n");
  const blob = new Blob([content], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `patch_log.txt`;
  a.click();
  URL.revokeObjectURL(url);
}

function setStatus(message, kind) {
  ui.status.textContent = message;
  ui.status.classList.remove("ok", "err");
  if (kind === "ok") ui.status.classList.add("ok");
  if (kind === "err") ui.status.classList.add("err");
}

function applyTheme(theme) {
  if (!theme) return;
  for (const [key, value] of Object.entries(theme)) {
    document.documentElement.style.setProperty(key, value);
  }
}

function renderInstructions(instructions) {
  ui.instructionsList.innerHTML = "";
  for (const item of instructions ?? []) {
    const li = document.createElement("li");
    li.textContent = String(item);
    ui.instructionsList.appendChild(li);
  }
}

function renderTargetVersions(targetVersions) {
  ui.targetVersion.innerHTML = "";
  for (const tv of targetVersions ?? []) {
    const opt = document.createElement("option");
    opt.value = String(tv.value);
    opt.textContent = tv.label ?? String(tv.value);
    ui.targetVersion.appendChild(opt);
  }
}

function configureHeader(game) {
  ui.gameTitle.textContent = game?.title ?? "Save Patcher";
  ui.gameSubtitle.textContent = game?.subtitle ?? "";
  document.title = ui.gameTitle.textContent;

  const url = game?.githubUrl ?? null;
  if (url) {
    ui.githubLink.href = url;
    ui.githubLink.style.display = "inline-flex";
    ui.githubLinkLabel.textContent = game?.githubLabel ?? "View source";
  } else {
    ui.githubLink.style.display = "none";
  }
}

function configureAdvancedMode(uiCfg) {
  const show = Boolean(uiCfg?.showAdvancedMode);
  ui.advancedBox.style.display = show ? "block" : "none";
  if (!show && ui.devType) {
    ui.devType.value = "0";
  }
}

function configureDropZoneHint(exts) {
  const pretty = (exts ?? []).join(", ");
  ui.dropZoneHint.textContent = pretty.length > 0 ? `Tap to select (${pretty})` : "Tap to select";

  if (Array.isArray(exts) && exts.length > 0) {
    ui.inputFile.setAttribute("accept", exts.join(","));
  }
}

async function initWasm() {
  ui.wasmState.textContent = "loading…";
  try {
    const modulePath = SITE_CONFIG?.wasm?.modulePath ?? "./pkg/gb_save_game.js";
    const mod = await import(modulePath);
    if (typeof mod.default === "function") {
      await mod.default();
    }
    wasm = {
      getSaveVersion: mod.get_save_version,
      patchSave: mod.patch_save,
      patchSaveWithLog: mod.patch_save_with_log,
    };

    if (typeof wasm.getSaveVersion !== "function") {
      throw new Error("WASM module is missing get_save_version(bytes)");
    }
    if (typeof wasm.patchSave !== "function") {
      throw new Error("WASM module is missing patch_save(bytes, targetVersion, devType)");
    }
    if (typeof wasm.patchSaveWithLog !== "function") {
      throw new Error("WASM module is missing patch_save_with_log(bytes, targetVersion, devType)");
    }

    ui.wasmState.textContent = "ready";
  } catch (err) {
    ui.wasmState.textContent = "failed";
    setStatus(`Failed to load game patcher: ${String(err?.message ?? err)}`, "err");
    wasm = null;
  }
}

function setDownloadBytes(bytes) {
  if (state.downloadUrl) {
    URL.revokeObjectURL(state.downloadUrl);
  }

  const blob = new Blob([bytes], { type: "application/octet-stream" });
  state.downloadUrl = URL.createObjectURL(blob);
  ui.downloadLink.href = state.downloadUrl;
  ui.downloadLink.style.display = "inline-flex";

  const outName = state.fileName ? `patched_${state.fileName}` : "patched_save.sav";
  ui.downloadLink.download = outName;
  ui.outputMeta.textContent = `${outName} (${bytes.length} bytes)`;
}

function clearDownload() {
  if (state.downloadUrl) {
    URL.revokeObjectURL(state.downloadUrl);
    state.downloadUrl = null;
  }
  ui.downloadLink.style.display = "none";
  ui.downloadLink.href = "#";
  ui.outputMeta.textContent = "(none)";
}

function updatePatchButton() {
  ui.patchBtn.disabled = !wasm || !state.bytes;
}

function parseDevType() {
  if (!ui.devType || ui.advancedBox.style.display === "none") return 0;
  const n = Number.parseInt(ui.devType.value ?? "0", 10);
  if (Number.isFinite(n) && n >= 0 && n <= 255) return n;
  return 0;
}

async function detectVersion() {
  if (!wasm || !state.bytes) {
    state.detectedVersion = null;
    ui.detectedVersion.textContent = "(unknown)";
    return;
  }

  try {
    const ver = wasm.getSaveVersion(state.bytes);
    state.detectedVersion = ver;
    ui.detectedVersion.textContent = String(ver);
  } catch (err) {
    state.detectedVersion = null;
    ui.detectedVersion.textContent = "(unknown)";
    logMessage(`Could not detect save version: ${String(err?.message ?? err)}`, "warn");
  }
}

async function patch() {
  if (!wasm || !state.bytes) return;

  clearDownload();
  setStatus("Patching…", undefined);

  const target = Number.parseInt(ui.targetVersion.value, 10);
  const devType = parseDevType();

  try {
    const outcome = wasm.patchSaveWithLog(state.bytes, target, devType);

    if (outcome?.logs?.length) {
      for (const entry of outcome.logs) {
        const msg = entry?.message ?? "";
        const lvl = entry?.level ?? "info";
        logMessage(msg, lvl);
      }
    }

    if (outcome?.error) {
      setStatus(outcome.error, "err");
      return;
    }

    if (!outcome?.bytes) {
      setStatus("Patch failed: no output bytes returned.", "err");
      return;
    }

    setDownloadBytes(outcome.bytes);
    setStatus("Done. Download your patched save.", "ok");
  } catch (err) {
    setStatus(`Patch failed: ${String(err?.message ?? err)}`, "err");
  }
}

function wireUi() {
  ui.clearLogBtn.addEventListener("click", () => {
    clearLog();
    setStatus("", undefined);
  });

  ui.downloadLogBtn.addEventListener("click", () => downloadLog());

  ui.patchBtn.addEventListener("click", () => patch());

  ui.dropZone.addEventListener("click", () => ui.inputFile.click());
  ui.dropZone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      ui.inputFile.click();
    }
  });

  ui.dropZone.addEventListener("dragover", (e) => {
    e.preventDefault();
    ui.dropZone.classList.add("dragover");
  });
  ui.dropZone.addEventListener("dragleave", () => ui.dropZone.classList.remove("dragover"));
  ui.dropZone.addEventListener("drop", async (e) => {
    e.preventDefault();
    ui.dropZone.classList.remove("dragover");
    const file = e.dataTransfer?.files?.[0] ?? null;
    if (file) {
      await loadFile(file);
    }
  });

  ui.inputFile.addEventListener("change", async () => {
    const file = ui.inputFile.files?.[0] ?? null;
    if (file) {
      await loadFile(file);
    }
  });
}

async function loadFile(file) {
  clearDownload();
  state.fileName = file.name;
  ui.inputMeta.textContent = `${file.name} (${file.size} bytes)`;

  try {
    const buf = await file.arrayBuffer();
    state.bytes = new Uint8Array(buf);
    setStatus("Save loaded.", undefined);
    await detectVersion();
  } catch (err) {
    state.bytes = null;
    setStatus(`Failed to read file: ${String(err?.message ?? err)}`, "err");
  }

  updatePatchButton();
}

async function boot() {
  configureHeader(SITE_CONFIG.game);
  applyTheme(SITE_CONFIG.theme);
  renderInstructions(SITE_CONFIG.instructions);
  renderTargetVersions(SITE_CONFIG.targetVersions);
  configureAdvancedMode(SITE_CONFIG.ui);
  configureDropZoneHint(SITE_CONFIG.acceptedExtensions);
  wireUi();

  clearLog();
  clearDownload();
  setStatus("", undefined);

  await initWasm();
  updatePatchButton();
}

boot();

function applyPatcherTheme(patcher) {
  const theme = patcher.theme ?? {};
  for (const [key, value] of Object.entries(theme)) {
    document.documentElement.style.setProperty(key, value);
  }
}

function clearDownload() {
  if (state.downloadUrl) {
    URL.revokeObjectURL(state.downloadUrl);
  }
  state.downloadUrl = null;
  ui.downloadLink.style.display = "none";
  ui.downloadLink.href = "#";
  ui.outputMeta.textContent = "(none)";
}

function targetVersionIsUsable() {
  return !ui.targetVersion.disabled && ui.targetVersion.options.length > 0;
}

function updatePatchButtonEnabled() {
  ui.patchBtn.disabled = !state.bytes || !targetVersionIsUsable();
}

function setAdvancedMode(enabled, { log } = { log: true }) {
  ui.advancedBox.style.display = enabled ? "block" : "none";
  if (!enabled) {
    ui.devType.value = "0";
  }

  if (log) {
    if (enabled) {
      logMessage("Advanced mode enabled (dev_type visible).");
    } else {
      logMessage("Advanced mode disabled (dev_type reset to 0).");
    }
  }
}

function safeInt(value, fallback) {
  const n = Number.parseInt(String(value), 10);
  return Number.isFinite(n) ? n : fallback;
}

function patchedFileName(originalName, targetVersion, devType) {
  const base = originalName?.replace(/\.[^/.]+$/, "") || "patched_save";
  const suffix = devType === 0 ? `v${targetVersion}` : `dev_type_${devType}`;
  return `${base}_${suffix}.sav`;
}

function canonicalVersionLabel(patcher, version) {
  const name = patcher.versionNames?.[version];
  return name ? `${version} (${name})` : String(version);
}

function setTargetVersionOptions({ patcher, detectedVersion, devType }) {
  ui.targetVersion.innerHTML = "";
  ui.targetVersion.disabled = false;

  const versionsAsc = [...patcher.supportedVersions].sort((a, b) => a - b);
  const max = Math.max(...versionsAsc);
  const haveDetected = typeof detectedVersion === "number" && Number.isFinite(detectedVersion);

  if (devType !== 0) {
    if (!haveDetected) {
      const option = document.createElement("option");
      option.value = String(max);
      option.textContent = canonicalVersionLabel(patcher, max);
      ui.targetVersion.appendChild(option);
    } else {
      const option = document.createElement("option");
      option.value = String(detectedVersion);
      option.textContent = canonicalVersionLabel(patcher, detectedVersion);
      ui.targetVersion.appendChild(option);
    }

    ui.targetVersion.disabled = true;
    ui.targetVersion.selectedIndex = 0;
    return;
  }

  if (!haveDetected) {
    const option = document.createElement("option");
    option.value = String(max);
    option.textContent = canonicalVersionLabel(patcher, max);
    ui.targetVersion.appendChild(option);
    ui.targetVersion.selectedIndex = 0;
    return;
  }

  const compatibleTargetsDesc = versionsAsc.filter((v) => v > detectedVersion).sort((a, b) => b - a);
  if (compatibleTargetsDesc.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = "No compatible targets";
    ui.targetVersion.appendChild(option);
    ui.targetVersion.disabled = true;
    ui.targetVersion.selectedIndex = 0;
    return;
  }

  for (const v of compatibleTargetsDesc) {
    const option = document.createElement("option");
    option.value = String(v);
    option.textContent = canonicalVersionLabel(patcher, v);
    ui.targetVersion.appendChild(option);
  }

  ui.targetVersion.selectedIndex = 0;
}

function currentDevType() {
  return ui.advancedToggle.checked ? safeInt(ui.devType.value, 0) : 0;
}

function refreshTargets() {
  const patcher = currentPatcher();
  const devType = currentDevType();
  setTargetVersionOptions({ patcher, detectedVersion: state.detectedVersion, devType });
  updatePatchButtonEnabled();

  let targetLog = null;
  if (devType !== 0) {
    const lockedTo = state.detectedVersion != null ? state.detectedVersion : Math.max(...patcher.supportedVersions);
    targetLog = `Fix patch mode (dev_type=${devType}): target locked to ${canonicalVersionLabel(patcher, lockedTo)}.`;
  } else if (state.detectedVersion != null) {
    if (ui.targetVersion.disabled) {
      targetLog = `No compatible migration targets for ${canonicalVersionLabel(patcher, state.detectedVersion)}.`;
    } else {
      const targets = [...ui.targetVersion.options].map((o) => o.textContent).join(", ");
      targetLog = `Compatible targets: ${targets}.`;
    }
  }

  if (targetLog && targetLog !== state.lastTargetLog) {
    logMessage(targetLog);
    state.lastTargetLog = targetLog;
  }

  if (state.bytes && state.detectedVersion != null && devType === 0 && ui.targetVersion.disabled) {
    setStatus("No compatible target versions for this save.", "err");
  }
}

async function onFileSelected(file) {
  clearDownload();

  const patcher = currentPatcher();

  if (!file) {
    state.bytes = null;
    state.fileName = null;
    state.detectedVersion = null;
    ui.inputMeta.textContent = "(none)";
    ui.detectedVersion.textContent = "(unknown)";
    refreshTargets();
    setStatus("Choose a .sav file to begin.", undefined);
    logMessage("Input cleared.");
    updatePatchButtonEnabled();
    return;
  }

  const buffer = await file.arrayBuffer();
  state.bytes = new Uint8Array(buffer);
  state.fileName = file.name;

  ui.inputMeta.textContent = `${file.name} (${state.bytes.length} bytes)`;
  logMessage(`Loaded file: ${file.name} (${state.bytes.length} bytes)`);

  try {
    const version = patcher.getSaveVersion(state.bytes);
    state.detectedVersion = version;
    ui.detectedVersion.textContent = canonicalVersionLabel(patcher, version);
    refreshTargets();

    setStatus("Ready to patch.", "ok");
    logMessage(`Detected save version: ${canonicalVersionLabel(patcher, version)}`);
  } catch (e) {
    state.detectedVersion = null;
    ui.detectedVersion.textContent = "(unknown)";
    refreshTargets();
    setStatus(String(e), "err");
    logMessage(`Error detecting save version: ${String(e)}`, "error");
  }

  updatePatchButtonEnabled();
}

async function onPatch() {
  clearDownload();

  if (!state.bytes) {
    setStatus("No input file selected.", "err");
    return;
  }

  const patcher = currentPatcher();

  const targetVersion = safeInt(ui.targetVersion.value, 10);
  const devType = currentDevType();

  ui.patchBtn.disabled = true;
  setStatus("Patching…", undefined);
  logMessage(`Patching started (target=${targetVersion}, dev_type=${devType})`);

  try {
    let outBytes;

    if (typeof patcher.patchSaveWithLog === "function") {
      const result = patcher.patchSaveWithLog(state.bytes, targetVersion, devType);
      const entries = Array.isArray(result?.logs) ? result.logs : [];

      for (const entry of entries) {
        const level = entry?.level === "error" ? "error" : entry?.level === "warn" ? "warn" : "info";
        const source = entry?.source ? String(entry.source) : "patch";
        const message = entry?.message ? String(entry.message) : "";
        if (message) {
          logMessage(`[${source}] ${message}`, level);
        }
      }

      if (!result?.ok) {
        const err = result?.error ? String(result.error) : "Patch failed.";
        throw new Error(err);
      }

      outBytes = result.bytes;
    } else {
      outBytes = patcher.patchSave(state.bytes, targetVersion, devType);
    }

    const blob = new Blob([outBytes], { type: "application/octet-stream" });
    state.downloadUrl = URL.createObjectURL(blob);

    ui.downloadLink.href = state.downloadUrl;
    ui.downloadLink.download = patchedFileName(state.fileName, targetVersion, devType);
    ui.downloadLink.style.display = "inline-flex";

    // In the upstream UI, a successful patch triggers an automatic download.
    // Keep the link visible as a fallback.
    logMessage("Triggering patched save download…");
    ui.downloadLink.click();
    logMessage("Patched save download triggered.");

    ui.outputMeta.textContent = `${outBytes.length} bytes`;
    setStatus("Patch complete.", "ok");
    logMessage(`Patching complete (${outBytes.length} bytes).`);
  } catch (e) {
    setStatus(String(e), "err");
    logMessage(`Patch failed: ${String(e)}`, "error");
  } finally {
    ui.patchBtn.disabled = false;
  }
}

function resetForPatcherChange() {
  clearDownload();
  state.detectedVersion = null;
  ui.detectedVersion.textContent = "(unknown)";
  ui.outputMeta.textContent = "(none)";
  setStatus("Choose a .sav file to begin.", undefined);
}

function onPatcherChanged() {
  state.patcherId = ui.patcherSelect.value;
  const patcher = currentPatcher();

  applyPatcherTheme(patcher);
  ui.patcherId.textContent = patcher.id;
  ui.patcherBlurb.textContent = patcher.blurb;
  refreshTargets();

  resetForPatcherChange();

  clearLog();
  logMessage(`Selected game: ${patcher.title}`);

  const file = ui.inputFile.files?.[0] ?? null;
  if (file) {
    onFileSelected(file);
  }
}

function setupDropZone() {
  ui.dropZone.addEventListener("click", () => {
    ui.inputFile.click();
  });

  ui.dropZone.addEventListener("dragover", (e) => {
    e.preventDefault();
    e.stopPropagation();
    ui.dropZone.classList.add("dragover");
  });

  ui.dropZone.addEventListener("dragleave", (e) => {
    e.preventDefault();
    e.stopPropagation();
    ui.dropZone.classList.remove("dragover");
  });

  ui.dropZone.addEventListener("drop", (e) => {
    e.preventDefault();
    e.stopPropagation();
    ui.dropZone.classList.remove("dragover");
    const file = e.dataTransfer?.files?.[0] ?? null;
    if (!file) return;

    logMessage(`File dropped: ${file.name}`);

    ui.inputFile.files = e.dataTransfer.files;
    ui.inputFile.dispatchEvent(new Event("change"));
  });
}

async function boot() {
  setStatus("Loading WASM…", undefined);
  clearLog();
  logMessage("Initializing WASM…");

  try {
    await init();
    ui.wasmState.textContent = "ready";
    setStatus("Choose a .sav file to begin.", undefined);
    logMessage("WASM initialized.");
  } catch (e) {
    ui.wasmState.textContent = "failed";
    setStatus(`WASM init failed: ${String(e)}`, "err");
    logMessage(`WASM init failed: ${String(e)}`, "error");
  }

  ui.patcherSelect.innerHTML = "";
  for (const patcher of patchers) {
    const option = document.createElement("option");
    option.value = patcher.id;
    option.textContent = patcher.title;
    ui.patcherSelect.appendChild(option);
  }
  ui.patcherSelect.value = state.patcherId;
  ui.patcherSelect.addEventListener("change", onPatcherChanged);

  ui.advancedToggle.addEventListener("change", () => {
    setAdvancedMode(ui.advancedToggle.checked);
    refreshTargets();
  });
  setAdvancedMode(false, { log: false });

  {
    const patcher = currentPatcher();
    applyPatcherTheme(patcher);
    ui.patcherId.textContent = patcher.id;
    ui.patcherBlurb.textContent = patcher.blurb;
    refreshTargets();
  }

  ui.devType.addEventListener("input", () => {
    if (ui.advancedToggle.checked) {
      logMessage(`dev_type set to ${safeInt(ui.devType.value, 0)}.`);
    }
    refreshTargets();
  });

  ui.downloadLogBtn.addEventListener("click", () => {
    downloadLog();
  });

  ui.clearLogBtn.addEventListener("click", () => {
    clearLog();
    logMessage("Log cleared.");
  });

  updateLogButtons();

  ui.inputFile.addEventListener("change", () => {
    const file = ui.inputFile.files?.[0] ?? null;
    onFileSelected(file);
  });

  setupDropZone();

  ui.patchBtn.addEventListener("click", () => {
    onPatch();
  });

  updatePatchButtonEnabled();
}

boot();

