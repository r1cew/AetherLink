<template>
  <div class="authorized-devices">
    <div v-if="servers.length === 0" class="empty-state">
      <p>Нет доступных устройств</p>
      <p class="hint">
        Перейдите на вкладку "Войти по QR" для подключения устройства
      </p>
    </div>

    <div v-else class="devices-container">
      <div class="devices-list">
        <div v-for="server in servers" :key="server.id" class="device-item">
          <div class="device-info">
            <h3 class="device-name">{{ server.name || "Устройство" }}</h3>
          </div>
          <div class="device-actions">
            <button class="connect-btn" @click.stop="selectServer(server)">
              Войти
            </button>

            <button class="delete-btn" @click.stop="removeServer(server.id)">
              <DeleteIcon />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DeleteIcon } from "lucide-vue-next";
import { useAetherLink } from "../../composables/useAetherLink";
const { selectServer, servers, removeServer } = useAetherLink();
</script>

<style scoped>
.authorized-devices {
  width: 100%;
  margin-top: 20px;
  padding: 0 16px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--text);
  opacity: 0.7;
}

.empty-state p {
  margin: 8px 0;
  font-size: 15px;
}

.empty-state .hint {
  font-size: 13px;
  opacity: 0.6;
}

.devices-container {
  background: none;
  border-radius: 13px;
  overflow: hidden;
}

.devices-header {
  padding: 8px 12px 12px 12px;
  border-bottom: 1px solid var(--red);
  margin-bottom: 16px;
}

.devices-title {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: var(--text);
  opacity: 0.8;
}

.devices-list {
  display: flex;
  width: 80vw;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  max-height: 33vh;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
}

.devices-list::-webkit-scrollbar {
  width: 3px;
}

.devices-list::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 10px;
}

.devices-list::-webkit-scrollbar-thumb {
  background: var(--red);
  border-radius: 10px;
}

.device-item {
  display: flex;
  align-items: center;
  gap: 50px;
  min-width: 320px;
  max-width: 320px;
  border: 1px solid var(--red);
  padding: 3vw;
  border-radius: 24px;
  flex-direction: row;
}

.device-item:active {
  transform: scale(0.98);
  background-color: rgba(255, 68, 68, 0.05);
}

.device-info {
  flex: 1;
  text-align: left;
  overflow: hidden;
  min-width: 0;
  margin-right: 12px;
}

.device-name {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.device-id {
  margin: 0;
  font-size: 11px;
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
}

.device-actions {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 10vw;
  flex-shrink: 0;
}

.connect-btn {
  font-family: inherit;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: none;
  color: var(--text);
  border: none;
  text-decoration: underline;
  border-radius: 40px;
  white-space: nowrap;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
}

.connect-btn:active {
  background: var(--red);
  transform: scale(0.95);
}

.connect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.delete-btn {
  background: none;
  border: none;
  margin-top: 5px;
  color: var(--text2);
  cursor: pointer;
}
</style>
