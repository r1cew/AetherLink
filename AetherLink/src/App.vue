<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ── состояние ──────────────────────────────────────────────────────────────────
const qrData   = ref("");
const devices  = ref<any[]>([]);
const profiles = ref<any[]>([]);
const devMode  = ref(false);
const log      = ref<string[]>([]);

// ── лог ───────────────────────────────────────────────────────────────────────
function addLog(msg: string) {
  log.value.unshift(`[${new Date().toLocaleTimeString()}] ${msg}`);
  if (log.value.length > 50) log.value.pop();
}

// ── загрузка ──────────────────────────────────────────────────────────────────
async function loadDevices() {
  try {
    devices.value = await invoke("get_devices");
    addLog(`Устройств: ${devices.value.length}`);
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

async function loadProfiles() {
  try {
    profiles.value = await invoke("get_profiles");
  } catch (e) { addLog(`Ошибка профилей: ${e}`) }
}

// ── паринг ────────────────────────────────────────────────────────────────────
async function generateQR() {
  try {
    qrData.value = await invoke("generate_pairing_qr");
    addLog("QR сгенерирован — действителен 120 сек");
  } catch (e) { addLog(`Ошибка QR: ${e}`) }
}

// ── устройства ────────────────────────────────────────────────────────────────
async function setMode(id: string, mode: string) {
  try {
    await invoke("set_device_mode", { deviceId: id, mode });
    addLog(`Режим устройства → ${mode}`);
    loadDevices();
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

async function removeDevice(id: string) {
  try {
    await invoke("remove_device", { deviceId: id });
    addLog("Устройство удалено");
    loadDevices();
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

// ── developer mode ────────────────────────────────────────────────────────────
async function toggleDevMode() {
  try {
    await invoke("set_developer_mode", { enabled: devMode.value });
    addLog(`Developer Mode: ${devMode.value ? "ВКЛ" : "ВЫКЛ"}`);
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

// ── профили ───────────────────────────────────────────────────────────────────
const newProfileName = ref("");
const newProfilePath = ref("");

async function createTestProfile() {
  if (!newProfileName.value || !newProfilePath.value) return;
  try {
    const id = await invoke("create_profile", {
      name: newProfileName.value,
      description: "Тестовый профиль",
      kind: { type: "run_bat", path: newProfilePath.value }
    });
    addLog(`Профиль создан: ${id}`);
    newProfileName.value = "";
    newProfilePath.value = "";
    loadProfiles();
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

async function deleteProfile(id: string) {
  try {
    await invoke("delete_profile", { profileId: id });
    addLog("Профиль удалён");
    loadProfiles();
  } catch (e) { addLog(`Ошибка: ${e}`) }
}

// ── init ──────────────────────────────────────────────────────────────────────
onMounted(async () => {
  loadDevices();
  loadProfiles();

  // Слушаем событие паринга с телефона
  await listen("device-paired", (event: any) => {
    addLog(`✅ Телефон привязан: ${event.payload.name}`);
    loadDevices();
  });

  addLog("Сервер запущен на :8080 | Beacon на :9999");
});
</script>

<template>
  <div class="app">
    <h1>⚡ AetherLink</h1>

    <!-- ── QR Паринг ─────────────────────────────────────── -->
    <section>
      <h2>📱 Паринг телефона</h2>
      <button @click="generateQR">Сгенерировать QR</button>
      <div v-if="qrData" class="qr-box">
        <p style="font-size:11px; word-break:break-all; opacity:0.7">{{ qrData }}</p>
        <p style="color:#4ade80">✓ Покажи этот JSON телефону (или отрисуй QR через библиотеку)</p>
      </div>
    </section>

    <!-- ── Устройства ────────────────────────────────────── -->
    <section>
      <h2>🔌 Устройства <span class="badge">{{ devices.length }}</span></h2>
      <button @click="loadDevices" class="sm">↻ Обновить</button>
      <div v-if="devices.length === 0" class="empty">Нет привязанных устройств</div>
      <div v-for="d in devices" :key="d.id" class="device-card">
        <span class="device-name">{{ d.name }}</span>
        <span :class="['mode-badge', d.mode]">{{ d.mode }}</span>
        <div class="device-actions">
          <button @click="setMode(d.id, 'safe')"       :class="{active: d.mode==='safe'}">Safe</button>
          <button @click="setMode(d.id, 'automation')" :class="{active: d.mode==='automation'}">Auto</button>
          <button @click="setMode(d.id, 'developer')"  :class="{active: d.mode==='developer'}">Dev</button>
          <button @click="removeDevice(d.id)" class="danger">✕</button>
        </div>
      </div>
    </section>

    <!-- ── Developer Mode ────────────────────────────────── -->
    <section>
      <h2>⚠️ Developer Mode</h2>
      <label class="toggle">
        <input type="checkbox" v-model="devMode" @change="toggleDevMode" />
        <span>{{ devMode ? "ВКЛ 🔓" : "ВЫКЛ 🔒" }}</span>
      </label>
      <p class="hint">Даёт доступ к shell. Только для доверенных устройств с режимом "developer".</p>
    </section>

    <!-- ── Профили ───────────────────────────────────────── -->
    <section>
      <h2>🤖 Automation Профили <span class="badge">{{ profiles.length }}</span></h2>
      <div class="new-profile">
        <input v-model="newProfileName" placeholder="Название (напр. Minecraft Server)" />
        <input v-model="newProfilePath" placeholder="Путь к .bat (напр. C:\srv\start.bat)" />
        <button @click="createTestProfile">+ Добавить</button>
      </div>
      <div v-if="profiles.length === 0" class="empty">Профилей нет</div>
      <div v-for="p in profiles" :key="p.id" class="profile-card">
        <div>
          <strong>{{ p.name }}</strong>
          <span v-if="p.description" class="hint"> — {{ p.description }}</span>
        </div>
        <code>{{ p.kind?.path ?? p.kind?.script ?? "..." }}</code>
        <button @click="deleteProfile(p.id)" class="danger sm">✕</button>
      </div>
    </section>

    <!-- ── Лог ───────────────────────────────────────────── -->
    <section>
      <h2>📋 Лог</h2>
      <div class="log-box">
        <div v-for="(line, i) in log" :key="i" class="log-line">{{ line }}</div>
      </div>
    </section>
  </div>
</template>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: 'Segoe UI', sans-serif;
  background: #0f0f13;
  color: #e2e2e2;
  font-size: 14px;
}

.app {
  max-width: 700px;
  margin: 0 auto;
  padding: 20px 16px 40px;
}

h1 { font-size: 22px; margin-bottom: 20px; color: #a78bfa; }
h2 { font-size: 15px; margin-bottom: 10px; color: #c4b5fd; }

section {
  background: #1a1a24;
  border: 1px solid #2a2a3a;
  border-radius: 10px;
  padding: 14px 16px;
  margin-bottom: 14px;
}

button {
  background: #2d2d42;
  color: #e2e2e2;
  border: 1px solid #3d3d5c;
  border-radius: 6px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.15s;
}
button:hover { background: #3d3d5c; }
button.active { background: #5b21b6; border-color: #7c3aed; color: #fff; }
button.danger { background: #3f1010; border-color: #7f2020; color: #f87171; }
button.danger:hover { background: #5c1a1a; }
button.sm { padding: 4px 8px; font-size: 12px; }

input {
  background: #111118;
  border: 1px solid #2a2a3a;
  border-radius: 6px;
  padding: 6px 10px;
  color: #e2e2e2;
  font-size: 13px;
  outline: none;
}
input:focus { border-color: #7c3aed; }

.badge {
  background: #312e81;
  color: #a5b4fc;
  border-radius: 10px;
  padding: 1px 8px;
  font-size: 11px;
  margin-left: 6px;
}

.qr-box {
  margin-top: 10px;
  background: #111118;
  border: 1px dashed #3d3d5c;
  border-radius: 8px;
  padding: 12px;
}

.empty { color: #555; font-style: italic; font-size: 13px; }

.device-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background: #111118;
  border-radius: 8px;
  margin-top: 6px;
  flex-wrap: wrap;
}
.device-name { flex: 1; font-weight: 500; }
.mode-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
}
.mode-badge.safe       { background: #14532d; color: #4ade80; }
.mode-badge.automation { background: #1e3a5f; color: #60a5fa; }
.mode-badge.developer  { background: #4c1d1d; color: #f87171; }

.device-actions { display: flex; gap: 4px; }

.toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
}
.toggle input { width: 16px; height: 16px; cursor: pointer; }

.hint { color: #666; font-size: 12px; margin-top: 6px; }

.new-profile {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.new-profile input:first-child { flex: 1; min-width: 150px; }
.new-profile input:nth-child(2) { flex: 2; min-width: 200px; }

.profile-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background: #111118;
  border-radius: 8px;
  margin-top: 6px;
}
.profile-card > div { flex: 1; }
.profile-card code {
  font-size: 11px;
  color: #888;
  background: #1a1a24;
  padding: 2px 6px;
  border-radius: 4px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-box {
  background: #080810;
  border-radius: 8px;
  padding: 10px;
  max-height: 180px;
  overflow-y: auto;
  font-family: 'Consolas', monospace;
  font-size: 12px;
}
.log-line {
  color: #888;
  padding: 2px 0;
  border-bottom: 1px solid #111;
}
.log-line:first-child { color: #c4b5fd; }
</style>
