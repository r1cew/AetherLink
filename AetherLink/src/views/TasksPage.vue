<template>
  <div class="app">
    <Navigation />

    <section class="app_part_2">
      <!-- ── Задачи ────────────────────────────────────── -->
      <section class="tasks-header-section">
        <div class="tasks-header">
          <h2>
            Ваши задачи <span class="badge">{{ totalTasksCount }}</span>
          </h2>
          <div class="header-buttons">
            <button class="sm" @click="">⟳ Обновить</button>
            <button class="sm primary" @click="openCreateTaskModal">
              + Создать задачу
            </button>
          </div>
        </div>
        <div class="tasks-container"></div>
      </section>

      <!-- ── Модальное окно создания задачи ─────────────────────────────────────── -->
      <div v-if="showModal" class="modal-overlay" @click="closeModal">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h3>Создать задачу</h3>
            <button class="modal-close" @click="closeModal">✕</button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Название задачи</label>
              <input
                v-model="newTask.name"
                type="text"
                placeholder="Введите название задачи"
                required
              />
            </div>
            <div class="form-group">
              <label>Описание</label>
              <textarea
                placeholder="Введите описание задачи"
                v-model="newTask.description"
              ></textarea>
            </div>
            <div class="form-group">
              <label>Тип задачи</label>
              <select v-model="newTask.type">
                <option value="run_bat">Запуск bat</option>
                <option value="run_exe">Запуск программы</option>
                <option value="power_shell">Запуск сервера (PowerShell)</option>
              </select>
            </div>

            <!-- Поле пути/директории: скрываем для power_shell -->
            <div class="form-group" v-if="newTask.type !== 'power_shell'">
              <label>Путь к файлу / Директория</label>
              <input
                v-model="newTask.path"
                type="text"
                placeholder="C:\path\to\file"
                required
              />
            </div>

            <!-- Дополнительное поле аргументов: только для run_exe -->
            <div class="form-group" v-if="newTask.type === 'run_exe'">
              <label>Аргументы (через пробел)</label>
              <input
                v-model="newTask.args"
                type="text"
                placeholder="--fullscreen --no-splash"
              />
            </div>

            <!-- Дополнительное поле скрипта: только для power_shell -->
            <div class="form-group" v-if="newTask.type === 'power_shell'">
              <label>PowerShell Скрипт</label>
              <textarea
                v-model="newTask.script"
                placeholder="Remove-Item $env:TEMP\* -Recurse -Force"
              ></textarea>
            </div>
          </div>
          <div class="modal-footer">
            <button class="cancel-btn" @click="closeModal">Отмена</button>
            <button class="create-btn" @click="handleCreateTask">
              Создать
            </button>
          </div>
        </div>
      </div>

      <!-- ── Лог ─────────────────────────────────────── -->
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
import { computed, ref } from "vue";
import "../assets/tasks.css";

const {
  log,
  addLog,
  loadDevices,
  createProfile,
  newProfileName,
  newProfilePath,
  newProfileDescription,
  newProfileType,
  newProfileArgs,
  newProfileScript,
} = useAetherLink();

// Модальное окно
const showModal = ref(false);

const newTask = ref({
  name: "",
  description: "",
  type: "run_bat",
  path: "",
  args: "", // Строка из инпута, преобразуем в массив перед отправкой
  script: "", // Для PowerShell
});

const handleCreateTask = async () => {
  // Базовая проверка на имя
  if (!newTask.value.name) return;

  // Динамическая валидация полей в зависимости от типа задачи
  if (newTask.value.type !== "power_shell" && !newTask.value.path) {
    addLog("Ошибка: Не указан путь к файлу/директории");
    return;
  }
  if (newTask.value.type === "power_shell" && !newTask.value.script) {
    addLog("Ошибка: Скрипт не может быть пустым");
    return;
  }

  // Заполняем реактивные переменные в useAetherLink
  newProfileName.value = newTask.value.name;
  newProfileDescription.value = newTask.value.description;
  newProfileType.value = newTask.value.type;
  newProfilePath.value = newTask.value.path;
  newProfileScript.value = newTask.value.script;

  // Парсим строку аргументов в массив строк (разбиваем по пробелам, удаляем лишние пустоты)
  newProfileArgs.value = newTask.value.args.trim()
    ? newTask.value.args.trim().split(/\s+/)
    : [];

  try {
    await createProfile();
    addLog(`Задача создана: ${newTask.value.name}`);
    closeModal();

    // Сброс формы в начальное состояние
    newTask.value = {
      name: "",
      description: "",
      type: "run_bat",
      path: "",
      args: "",
      script: "",
    };
  } catch (e) {
    addLog(`Ошибка при создании: ${e}`);
  }
};

const totalTasksCount = computed(() => {
  return 1; // Ваша логика подсчета количества задач
});

const openCreateTaskModal = () => {
  showModal.value = true;
  loadDevices();
};

const closeModal = () => {
  showModal.value = false;
};
</script>
