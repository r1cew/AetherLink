<template>
  <Teleport to="body">
    <div
      v-if="showColorPicker"
      class="color-picker-overlay"
      @click.self="close"
    >
      <div class="color-picker-panel">
        <div class="panel-header">
          <div class="header-title">
            <Palette class="palette-icon" />
            <h3>Изменить тему</h3>
          </div>
          <button class="close-btn" @click="close">✕</button>
        </div>

        <div class="color-grid">
          <div class="color-item">
            <label>Основной цвет:</label>
            <input
              type="color"
              :value="colors.red"
              @input="handleColorChange('red', $event)"
            />
            <span class="color-value">{{ colors.red }}</span>
          </div>

          <div class="color-item">
            <label>Акцентные цвета:</label>
            <input
              type="color"
              :value="colors.accent"
              @input="handleColorChange('accent', $event)"
            />
            <span class="color-value">{{ colors.accent }}</span>
          </div>

          <div class="color-item">
            <label>Акцентные цвета 2:</label>
            <input
              type="color"
              :value="colors.accent2"
              @input="handleColorChange('accent2', $event)"
            />
            <span class="color-value">{{ colors.accent2 }}</span>
          </div>

          <div class="color-item">
            <label>Цвет текста:</label>
            <input
              type="color"
              :value="colors.text"
              @input="handleColorChange('text', $event)"
            />
            <span class="color-value">{{ colors.text }}</span>
          </div>

          <div class="color-item">
            <label>Фоновое изображение:</label>
            <input
              type="file"
              accept="image/*"
              @change="handleImageUpload"
              style="display: none"
              ref="fileInput"
            />
            <button @click="triggerFileInput" class="upload-btn">
              {{ backgroundImage ? "Изменить фото" : "Загрузить фото" }}
            </button>
            <button
              v-if="backgroundImage"
              @click="removeBackground"
              class="remove-btn"
            >
              Удалить
            </button>
          </div>
        </div>

        <div class="panel-footer">
          <button class="reset-btn" @click="resetTheme">По умолчанию</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { Palette } from "lucide-vue-next";
import { useThemeCustomizer } from "../../composables/useThemeCustomizer";

import { ref } from "vue";

const {
  showColorPicker,
  colors,
  updateColor,
  resetTheme,
  backgroundImage,
  setBackgroundImage,
  removeBackgroundImage,
} = useThemeCustomizer();

const fileInput = ref<HTMLInputElement | null>(null);

const triggerFileInput = () => {
  fileInput.value?.click();
};

const handleImageUpload = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    try {
      await setBackgroundImage(file);
    } catch (error) {
      console.error("Failed to upload image:", error);
    }
  }
};

const handleColorChange = (key: keyof typeof colors, event: Event) => {
  const target = event.target as HTMLInputElement;
  updateColor(key, target.value);
};

const close = () => {
  showColorPicker.value = false;
};

const removeBackground = () => {
  removeBackgroundImage();
  if (fileInput.value) {
    fileInput.value.value = "";
  }
};
</script>

<style scoped>
.color-picker-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(4px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.color-picker-panel {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 24px;
  max-width: 500px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.palette-icon {
  width: 24px;
  height: 24px;
  color: var(--red);
  stroke-width: 1.5;
}

.panel-header h3 {
  margin: 0;
  color: var(--text);
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text2);
  font-size: 20px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg3);
  color: var(--red);
}

.color-grid {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
}

.color-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  background: var(--bg);
  border-radius: 8px;
}

.color-item label {
  flex: 1;
  font-size: 13px;
  color: var(--text2);
}

.color-item input {
  width: 50px;
  height: 40px;
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg2);
}

.color-value {
  width: 70px;
  font-size: 11px;
  font-family: monospace;
  color: var(--text2);
}

.panel-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid var(--border);
}

.reset-btn {
  padding: 10px;
  background: var(--red);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
}

.reset-btn:hover {
  background: var(--red);
  border-color: var(--red);
  color: white;
}

.note {
  text-align: center;
  font-size: 12px;
  color: var(--text2);
}

.upload-btn,
.remove-btn {
  padding: 8px 16px;
  background: var(--red);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.upload-btn:hover {
  background: var(--red);
  border-color: var(--accent);
  color: white;
}

.remove-btn:hover {
  background: var(--red);
  border-color: var(--red);
  color: white;
}
</style>
