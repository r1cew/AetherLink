<template>
  <div class="app">
    <section class="app_part_1" v-if="addPhone == true">
      <h1>AetherLink</h1>

      <!-- ── QR Паринг ─────────────────────────────────────── -->
      <section>
        <Qrcode
          class="qrcode"
          v-if="qrData"
          :value="qrData"
          :size="200"
          render-as="svg"
        />
        <div v-if="!qrData" class="qr-without">
          <p>Сгенерируйте QR</p>
        </div>
        <button @click="generateQR">Сгенерировать QR</button>

        <p style="color: #4ade80" v-if="qrData">✓ Отсканируй этот QR код</p>
        <span v-if="qrData">
          {{
            formatTime(timeLeft) > "0"
              ? `Действует: ${formatTime(timeLeft)}`
              : "Действие QR кода истёк"
          }}
        </span>
        <span
          v-if="qrData"
          class="qr_help"
          style="cursor: pointer"
          @click="showJson()"
          >Не работает QR?</span
        >
        <div v-if="jsonCheck == true && qrData" class="qr-box">
          <p style="font-size: 11px; word-break: break-all; opacity: 0.7">
            {{ qrData }}
          </p>
        </div>
        <button
          style="background: none; color: gray"
          @mouseover="(e) => (e.target.style.color = 'white')"
          @mouseout="(e) => (e.target.style.color = 'gray')"
          @click="addPhone = !addPhone"
        >
          Вернуться назад
        </button>
      </section>
    </section>

    <Navigation v-if="addPhone == false" />

    <section class="app_part_2">
      <!-- ── Устройства ────────────────────────────────────── -->
      <section class="devices">
        <div class="add-user">
          <h2>
            Устройства <span class="badge">{{ devices.length }}</span>
          </h2>
          <div class="header-buttons">
            <button @click="loadDevices" class="sm">↻ Обновить</button>
            <button class="sm primary" @click="addPhone = !addPhone">
              {{ addPhone ? "Скрыть QR" : "Добавить устройство" }}
            </button>
          </div>
        </div>

        <div v-if="devices.length === 0" class="empty">
          Нет привязанных устройств
        </div>
        <div v-for="device in devices" :key="device.id" class="device-card">
          <span class="device-name">{{ device.name }}</span>
          <span :class="['mode-badge', device.mode]">{{ device.mode }}</span>
          <div class="device-actions">
            <button
              @click="setMode(device.id, 'default')"
              :class="{ active: device.mode === 'default' }"
            >
              Default
            </button>
            <button
              @click="setMode(device.id, 'developer')"
              :class="{ active: device.mode === 'developer' }"
            >
              Dev
            </button>
            <button @click="removeDevice(device.id)" class="danger">✕</button>
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
import Navigation from "../components/Navigation.vue";
import { useAetherLink } from "../composables/useAetherLink";
import Qrcode from "qrcode.vue";
import "../assets/style.css";

// Примерно такое должно прийти с запроса
const fakeDataBase = [
  {
    id: 1,
    device: "Phone 1",
    tasks: [
      {
        id: 1,
        name: "Ютуб",
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
  log,
  addPhone,
  jsonCheck,
  generateQR,
  timeLeft,
  formatTime,
  loadDevices,
  setMode,
  removeDevice,
  showJson,
} = useAetherLink();
</script>
