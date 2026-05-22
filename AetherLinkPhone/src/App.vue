<template>
    <!-- ЭКРАН 1: СПИСОК СЕРВЕРОВ -->
    <main v-if="screen === 'servers'" class="shell">
        <div class="topbar">
            <span class="app-title">⚡ AetherLink</span>
            <button class="icon-btn" @click="screen = 'pair'">＋</button>
        </div>

        <div v-if="servers.length === 0" class="empty-state">
            <p>Нет привязанных ПК</p>
            <button class="btn-primary" @click="screen = 'pair'">
                Привязать ПК
            </button>
        </div>

        <div v-else class="server-list">
            <div
                v-for="s in servers"
                :key="s.id"
                class="server-card"
                @click="selectServer(s)"
            >
                <div class="server-icon">🖥</div>
                <div class="server-info">
                    <div class="server-name">{{ s.name }}</div>
                    <div class="server-addr">{{ s.ip }}:{{ s.port }}</div>
                </div>
                <div class="chevron">›</div>
            </div>
        </div>
    </main>

    <!-- ЭКРАН 2: ПАРИНГ -->
    <main v-else-if="screen === 'pair'" class="shell">
        <!-- Оверлей сканера камеры поверх WebView -->
        <div v-if="isScanning" class="camera-overlay">
            <div class="scanner-hint">Наведите камеру на QR-код на ПК</div>
            <div class="scanner-target"></div>
            <button
                class="btn-primary"
                style="background: var(--red)"
                @click="stopQrScan"
            >
                Отмена
            </button>
        </div>

        <div class="topbar">
            <button class="icon-btn" @click="screen = 'servers'">‹</button>
            <span class="app-title">Привязать ПК</span>
            <span />
        </div>

        <div class="pair-screen">
            <div class="pair-icon" style="cursor: pointer" @click="startQrScan">
                📷
            </div>
            <p class="pair-hint">
                Нажмите кнопку ниже и наведите камеру на QR-код — подключение
                произойдёт автоматически.
            </p>

            <label class="field-label">Имя телефона (видно на ПК)</label>
            <input v-model="phoneName" class="text-input" />

            <label class="field-label">Название ПК (видно тебе)</label>
            <input v-model="pcName" class="text-input" />

            <button
                class="btn-primary"
                style="margin-bottom: 4px"
                :disabled="loading"
                @click="startQrScan"
            >
                {{ loading ? "⏳ Подключение…" : "[📷] Сканировать QR" }}
            </button>

            <hr style="border-color: var(--border); margin: 8px 0" />

            <label class="field-label">Или вставьте JSON вручную:</label>
            <textarea
                v-model="qrText"
                class="qr-input"
                :placeholder="`{\&quot;ip\&quot;:\&quot;192.168.1.5\&quot;,\&quot;port\&quot;:8080}`"
                :rows="3"
            />
            <button
                class="btn-primary"
                :disabled="loading || !qrText.trim()"
                @click="pair"
            >
                {{ loading ? "⏳ Подключение…" : "Привязать вручную" }}
            </button>

            <div v-if="log" class="log-box">{{ log }}</div>
        </div>
    </main>

    <!-- ЭКРАН 3: УПРАВЛЕНИЕ -->
    <main v-else class="shell">
        <div class="topbar">
            <button class="icon-btn" @click="screen = 'servers'">‹</button>
            <span class="app-title">{{ active?.name }}</span>
            <button class="icon-btn" title="Найти по beacon" @click="discover">
                📡
            </button>
        </div>

        <!-- Default Mode (Объединенный блок управления) -->
        <section class="section">
            <div class="section-title">⚡ Управление ПК</div>

            <div class="grid-2">
                <button class="ctrl-btn danger" @click="safe('shutdown')">
                    <span>⏻</span>Выключить
                </button>
                <button class="ctrl-btn" @click="safe('sleep')">
                    <span>🌙</span>Сон
                </button>
                <button class="ctrl-btn" @click="safe('lock')">
                    <span>🔒</span>Блокировка
                </button>
            </div>

            <div class="section-subtitle">Звук</div>
            <div class="grid-3">
                <button class="ctrl-btn sm" @click="safe('volume_down')">
                    🔉
                </button>
                <button
                    class="ctrl-btn sm"
                    @click="safe('volume_set', { level: 50 })"
                >
                    50%
                </button>
                <button class="ctrl-btn sm" @click="safe('volume_up')">
                    🔊
                </button>
            </div>

            <div class="section-subtitle">Медиа</div>
            <div class="grid-4">
                <button class="ctrl-btn sm" @click="safe('media_prev')">
                    ⏮
                </button>
                <button class="ctrl-btn sm" @click="safe('media_play')">
                    ⏯
                </button>
                <button class="ctrl-btn sm" @click="safe('media_pause')">
                    ⏸
                </button>
                <button class="ctrl-btn sm" @click="safe('media_next')">
                    ⏭
                </button>
            </div>
        </section>

        <!-- Автоматизация (Профили) -->
        <section v-if="profiles.length > 0" class="section">
            <div class="section-title">🤖 Автоматизация</div>
            <button
                v-for="p in profiles"
                :key="p.id"
                class="profile-btn"
                @click="runProfile(p.id, p.name)"
            >
                <span class="profile-name">{{ p.name }}</span>
                <span v-if="p.description" class="profile-desc">{{
                    p.description
                }}</span>
            </button>
        </section>

        <!-- Консоль разработчика -->
        <section class="section">
            <div class="section-title">🛠 Разработчик Shell</div>
            <div class="shell-row">
                <input
                    v-model="shellCmd"
                    class="text-input shell-input"
                    placeholder="powershell команда..."
                    @keydown.enter="runShell"
                />
                <button class="btn-send" :disabled="loading" @click="runShell">
                    ▶
                </button>
            </div>
        </section>

        <!-- Всплывающий лог -->
        <div v-if="log" class="log-box" @click="log = ''">
            <pre>{{ log }}</pre>
            <small>тап чтобы закрыть</small>
        </div>

        <div v-if="loading" class="loading-bar" />
    </main>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
    scan,
    cancel,
    Format,
    checkPermissions,
    requestPermissions,
} from "@tauri-apps/plugin-barcode-scanner";

// ── Интерфейсы типов ──────────────────────────────────────────────────────────
interface Server {
    id: string;
    name: string;
    ip: string;
    port: number;
}

interface Profile {
    id: string;
    name: string;
    description?: string;
}

type Screen = "servers" | "pair" | "control";

// ── Реактивные переменные (Стейт) ──────────────────────────────────────────────
const screen = ref<Screen>("servers");
const servers = ref<Server[]>([]);
const active = ref<Server | null>(null);
const profiles = ref<Profile[]>([]);
const log = ref("");
const loading = ref(false);
const isScanning = ref(false);

const qrText = ref("");
const phoneName = ref("Мой телефон");
const pcName = ref("Домашний ПК");
const shellCmd = ref("");

// Вспомогательная функция для логов
function msg(text: string) {
    log.value = text;
}

// ── Логика управления прозрачностью body при сканировании ──────────────────────
watch(isScanning, (newValue) => {
    if (newValue) {
        document.body.classList.add("scanning-active");
    } else {
        document.body.classList.remove("scanning-active");
    }
});

// На случай внезапного демонтирования компонента чистим класс
onUnmounted(() => {
    document.body.classList.remove("scanning-active");
});

// ── Загрузка серверов ─────────────────────────────────────────────────────────
async function loadServers() {
    try {
        const list = await invoke<Server[]>("get_servers");
        servers.value = list;
    } catch (e) {
        msg(`Ошибка: ${e}`);
    }
}

onMounted(() => {
    loadServers();
});

// ── Загрузка профилей автоматизации ───────────────────────────────────────────
async function loadProfiles(server: Server) {
    try {
        const list = await invoke<Profile[]>("list_profiles", {
            serverId: server.id,
        });
        profiles.value = Array.isArray(list) ? list : [];
    } catch {
        profiles.value = [];
    }
}

// ── Действия с серверами ──────────────────────────────────────────────────────
function selectServer(s: Server) {
    active.value = s;
    log.value = "";
    loadProfiles(s);
    screen.value = "control";
}

// ── Авто-подключение после сканирования ───────────────────────────────────────
async function autoConnect(content: string) {
    loading.value = true;
    msg("⏳ Подключение...");
    try {
        const id = await invoke<string>("pair_with_qr", {
            qrJson: content.trim(),
            name: phoneName.value,
            nickname: pcName.value,
        });

        await loadServers();
        const newServer = servers.value.find((s) => s.id === id);

        if (newServer) {
            active.value = newServer;
            log.value = "";
            await loadProfiles(newServer);
            screen.value = "control";
        } else {
            screen.value = "servers";
        }
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}

// ── Работа с камерой (QR) ─────────────────────────────────────────────────────
async function startQrScan() {
    log.value = "";
    try {
        let permission = await checkPermissions();
        if (permission !== "granted") {
            msg("Запрашиваю доступ к камере...");
            permission = await requestPermissions();
        }
        if (permission !== "granted") {
            msg("❌ Доступ к камере отклонен. Включите в настройках телефона.");
            return;
        }

        isScanning.value = true;
        const result = await scan({ formats: [Format.QRCode] });

        isScanning.value = false;

        if (result?.content) {
            await autoConnect(result.content);
        }
    } catch (e) {
        msg(`Ошибкаカメラ: ${e}`);
    } finally {
        isScanning.value = false;
    }
}

async function stopQrScan() {
    try {
        await cancel();
    } catch {}
    isScanning.value = false;
}

// ── Ручной паринг через JSON ──────────────────────────────────────────────────
async function pair() {
    if (!qrText.value.trim()) {
        msg("Сначала отсканируй QR или вставь JSON");
        return;
    }
    loading.value = true;
    try {
        const id = await invoke<string>("pair_with_qr", {
            qrJson: qrText.value.trim(),
            name: phoneName.value,
            nickname: pcName.value,
        });
        msg(`✅ Привязано! ID: ${id.slice(0, 8)}…`);
        qrText.value = "";
        await loadServers();
        screen.value = "servers";
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}

// ── Вызовы системных команд (Default Mode) ────────────────────────────────────
async function safe(command: string, params?: object) {
    if (!active.value) return;
    loading.value = true;
    try {
        const res = await invoke("send_safe", {
            serverId: active.value.id,
            command,
            params: params ?? null,
        });
        msg(`✅ ${command}: ${JSON.stringify(res)}`);
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}

// ── Запуск профилей автоматизации ─────────────────────────────────────────────
async function runProfile(profileId: string, name: string) {
    if (!active.value) return;
    loading.value = true;
    try {
        await invoke("send_run_profile", {
            serverId: active.value.id,
            profileId,
        });
        msg(`✅ Запущено: ${name}`);
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}

// ── PowerShell терминал ───────────────────────────────────────────────────────
async function runShell() {
    if (!active.value || !shellCmd.value.trim()) return;
    loading.value = true;
    const currentCmd = shellCmd.value;
    try {
        const res = await invoke<string>("send_shell", {
            serverId: active.value.id,
            cmd: currentCmd,
            shell: "powershell",
        });
        msg(`> ${currentCmd}\n${res}`);
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}

// ── Локальный Beacon поиск ПК ─────────────────────────────────────────────────
async function discover() {
    if (!active.value) return;
    loading.value = true;
    msg("Ищу ПК в сети...");
    try {
        const ip = await invoke<string>("discover_and_update", {
            serverId: active.value.id,
        });
        msg(`✅ Найден: ${ip}`);
        await loadServers();
    } catch (e) {
        msg(`❌ ${e}`);
    } finally {
        loading.value = false;
    }
}
</script>
