<template>
  <div class="app">
    <section class="app_part_1">
      <h1>AetherLink</h1>

      <!-- ── QR Паринг ─────────────────────────────────────── -->
      <section>
        <Qrcode :value="qrData" :size="200" render-as="svg" />
        <button @click="generateQR">Сгенерировать QR</button>
        <p style="color: #4ade80">✓ Отсканируй этот QR код</p>
        <span class="qr_help" style="cursor: pointer" @click="showJson()"
          >Не работает QR?</span
        >
        <div v-if="jsonCheck == true && qrData" class="qr-box">
          <p style="font-size: 11px; word-break: break-all; opacity: 0.7">
            {{ qrData }}
          </p>
        </div>
      </section>
    </section>

    <section class="app_part_2">
      <!-- ── Устройства ────────────────────────────────────── -->
      <section class="devices">
        <h2>
          Устройства <span class="badge">{{ devices.length }}</span>
          <button @click="loadDevices" class="sm">↻ Обновить</button>
        </h2>
        <div></div>

        <div v-if="devices.length === 0" class="empty">
          Нет привязанных устройств
        </div>
        <div v-for="d in devices" :key="d.id" class="device-card">
          <span class="device-name">{{ d.name }}</span>
          <span :class="['mode-badge', d.mode]">{{ d.mode }}</span>
          <div class="device-actions">
            <button
              @click="setMode(d.id, 'Default')"
              :class="{ active: d.mode === 'Default' }"
            >
              Default
            </button>
            <button
              @click="setMode(d.id, 'developer')"
              :class="{ active: d.mode === 'developer' }"
            >
              Dev
            </button>
            <button @click="removeDevice(d.id)" class="danger">✕</button>
          </div>
        </div>
      </section>

      <section>
        <h2>Ваши задачи</h2>
        <div class="tasks-container">
          <!-- Перебираем все устройства -->
          <div
            v-for="device in fakeDataBase"
            :key="device.id"
            class="device-tasks"
          >
            <h3 class="device-title">{{ device.device }}</h3>

            <!-- Если есть задачи, показываем их -->
            <div
              v-if="device.tasks && device.tasks.length > 0"
              class="tasks-list"
            >
              <div
                v-for="task in device.tasks"
                :key="task.id"
                class="task-item"
              >
                <span class="task-name">{{ task.name }}</span>
                <button class="task-btn">Выполнить</button>
              </div>
            </div>

            <!-- Если задач нет -->
            <div v-else class="empty-tasks">Нет задач для этого устройства</div>
          </div>
        </div>
      </section>

      <!-- ── Лог ───────────────────────────────────────────── -->
      <section>
        <h2>Лог</h2>
        <div class="log-box">
          <div v-for="(line, i) in log" :key="i" class="log-line">
            {{ line }}
          </div>
        </div>
      </section>
    </section>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "./composables/useAetherLink";
import Qrcode from "qrcode.vue";
import "./assets/style.css";

// Примерно такое должно прийти с запроса
const fakeDataBase = [
  {
    id: 1,
    device: "Phone 1",
    tasks: [
      {
        id: 1,
        name: "Ютуб",
        // И так далее, Мейби тут скрипт может быть запуска. Я хз. Нужно как то организовать тоже
      },
      {
        id: 2,
        name: "Стим",
      },
    ],
  },
  {
    id: 2,
    device: "Phone 2",
    tasks: [
      {
        id: 1,
        name: "Роблокс",
        // И так далее
      },
      {
        id: 2,
        name: "Аниме",
      },
    ],
  },
];

const {
  qrData,
  devices,
  profiles,
  devMode,
  log,
  jsonCheck,
  generateQR,
  loadDevices,
  setMode,
  removeDevice,
  toggleDevMode,
  createTestProfile,
  deleteProfile,
  showJson,
} = useAetherLink();
</script>
