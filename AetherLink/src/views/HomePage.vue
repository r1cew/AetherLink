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
              @click="setMode(device.id, 'default', device.name)"
              :class="{ active: device.mode === 'default' }"
            >
              Default
            </button>
            <button
              @click="setMode(device.id, 'developer', device.name)"
              :class="{ active: device.mode === 'developer' }"
            >
              Dev
            </button>
            <button @click="removeDevice(device.id)" class="danger">✕</button>
          </div>
        </div>
      </section>
      <section class="startup">
        <h2>Автозапуск <span class="badge">Выкл</span></h2>
        <div class="toggle-container">
          <input
            type="checkbox"
            @change="addToStartup"
            id="toggleSwitch"
            class="toggle-input"
          />
          Тестовая кнопка выключения(пока отсутствует отслеживание)
          <button @click="removeFromStartup">выкл</button>
          <label for="toggleSwitch" class="toggle-slider"></label>
        </div>
      </section>
      <Logs />
    </section>
  </div>
</template>

<script setup lang="ts">
import Navigation from "../components/Navigation.vue";
import { useAetherLink } from "../composables/useAetherLink";
import Qrcode from "qrcode.vue";
import Logs from "../components/Logs.vue";
import "../assets/style.css";
import { onMounted } from "vue";

const {
  qrData,
  devices,
  addPhone,
  jsonCheck,
  generateQR,
  timeLeft,
  formatTime,
  addToStartup,
  removeFromStartup,
  loadDevices,
  setMode,
  removeDevice,
  showJson,
} = useAetherLink();

onMounted(() => {
  loadDevices();
});
</script>
