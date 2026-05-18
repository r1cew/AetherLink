<template>
  <div class="app">
    <section class="app_part_1">
      <Navigation />
    </section>

    <section class="app_part_2">
      <!-- ── Задачи ────────────────────────────────────── -->
      <section class="tasks-header-section">
        <div class="tasks-header">
          <h2>
            Ваши задачи <span class="badge">{{ totalTasksCount }}</span>
          </h2>
          <button class="sm" @click="refreshTasks">⟳ Обновить</button>
        </div>
      </section>

      <section class="tasks-list-section">
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
                <button
                  class="task-btn"
                  @click="executeTask(device.device, task.name)"
                >
                  Выполнить
                </button>
              </div>
            </div>

            <!-- Если задач нет -->
            <div v-else class="empty-tasks">Нет задач для этого устройства</div>
          </div>
        </div>
      </section>

      <!-- ── Лог выполнения ─────────────────────────────────────── -->
      <section>
        <h2>Лог выполнения</h2>
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
import { computed } from "vue";

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

const { log, addLogEntry } = useAetherLink();

// Подсчет общего количества задач
const totalTasksCount = computed(() => {
  return fakeDataBase.reduce((total, device) => {
    return total + (device.tasks?.length || 0);
  }, 0);
});

const executeTask = (device: string, task: string) => {
  // Логика выполнения задачи
  if (addLogEntry) {
    addLogEntry(`▶ Выполняется задача "${task}" на устройстве ${device}`);
  }

  // Здесь будет вызов API или другой логики
  setTimeout(() => {
    if (addLogEntry) {
      addLogEntry(`✅ Задача "${task}" выполнена на ${device}`);
    }
  }, 1000);
};

const refreshTasks = () => {
  if (addLogEntry) {
    addLogEntry(`🔄 Обновление списка задач...`);
  }
  // Здесь будет логика обновления задач
  setTimeout(() => {
    if (addLogEntry) {
      addLogEntry(`✅ Список задач обновлен`);
    }
  }, 500);
};
</script>

<style scoped>
/* Сохраняем ту же структуру что и в главной странице */
.app {
  max-width: 1400px;
  margin: 0 auto;
  min-height: 90vh;
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 2rem;
  padding: 2rem;
}

.app_part_1 {
  display: flex;
  flex-direction: column;
  align-items: center;
  max-height: 90vh;
  justify-content: flex-start;
  gap: 2rem;
  padding: 2rem;
  border-radius: 24px;
  backdrop-filter: blur(10px);
  position: sticky;
  top: 2rem;
}

.app_part_2 {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  max-height: 90vh;
  overflow-y: auto;
  padding-right: 0.5rem;
}

.app_part_2::-webkit-scrollbar {
  width: 6px;
}

.app_part_2::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}

.app_part_2::-webkit-scrollbar-thumb {
  background: rgba(216, 50, 60, 0.4);
  border-radius: 3px;
}

.app_part_2 section {
  background: rgba(255, 255, 255, 0.04);
  border-radius: 20px;
  padding: 1.25rem;
  backdrop-filter: blur(10px);
  transition: all 0.2s ease;
}

.app_part_2 h2 {
  font-size: 18px;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #ffffff;
}

.badge {
  background: rgba(216, 50, 60, 0.3);
  padding: 0.2rem 0.5rem;
  border-radius: 20px;
  font-size: 12px;
  font-weight: normal;
}

.tasks-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

button {
  background: rgba(216, 50, 60, 0.15);
  border: 1px solid rgba(216, 50, 60, 0.3);
  color: #ffffff;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
}

button:hover {
  background: rgba(216, 50, 60, 0.3);
  border-color: #d8323c;
  transform: translateY(-1px);
}

button:active {
  transform: translateY(0);
}

button.sm {
  padding: 0.25rem 0.75rem;
  font-size: 12px;
}

.tasks-container {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.device-tasks {
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  padding: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: all 0.2s ease;
}

.device-tasks:hover {
  border-color: rgba(216, 50, 60, 0.2);
  background: rgba(255, 255, 255, 0.05);
}

.device-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 0.75rem;
  color: #d8323c;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.task-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.task-item:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateX(4px);
}

.task-name {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

.task-btn {
  padding: 0.25rem 0.75rem;
  font-size: 12px;
  background: rgba(74, 222, 128, 0.15);
  border-color: rgba(74, 222, 128, 0.3);
}

.task-btn:hover {
  background: rgba(74, 222, 128, 0.3);
  border-color: #4ade80;
}

.empty-tasks {
  text-align: center;
  padding: 1rem;
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
  font-style: italic;
}

.log-box {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 12px;
  padding: 0.75rem;
  max-height: 250px;
  overflow-y: auto;
  font-family: monospace;
  font-size: 12px;
}

.log-line {
  padding: 0.25rem 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  color: rgba(255, 255, 255, 0.8);
}

.log-line:first-child {
  color: #4ade80;
}

@media (max-width: 768px) {
  .app {
    grid-template-columns: 1fr;
    gap: 1rem;
    padding: 1rem;
  }

  .app_part_1 {
    position: static;
    max-height: none;
    padding: 1rem;
  }

  .app_part_2 {
    max-height: none;
  }
}
</style>
