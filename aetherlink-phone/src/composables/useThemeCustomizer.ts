// composables/useThemeCustomizer.ts
import { ref, reactive, readonly } from "vue";

interface ThemeColors {
  red: string;
  accent: string;
  accent2: string;
  text: string;
}

const defaultTheme: ThemeColors = {
  red: "#d8323c",
  accent: "rgba(216, 50, 61, 0.507)",
  accent2: "#e2e2f0",
  text: "#e2e2f0",
};

// Create singleton state outside composable
const showColorPicker = ref(false);
const colors = reactive<ThemeColors>({ ...defaultTheme });
const backgroundImage = ref<string | null>(null);

export function useThemeCustomizer() {
  const applyTheme = (theme: ThemeColors) => {
    const root = document.documentElement;
    // Map color keys to CSS variable names
    const cssVarMap: Record<keyof ThemeColors, string> = {
      red: "--red",
      accent: "--accent",
      accent2: "--accent2",
      text: "--text",
    };

    Object.entries(theme).forEach(([key, value]) => {
      const cssKey = cssVarMap[key as keyof ThemeColors];
      if (cssKey) {
        root.style.setProperty(cssKey, value);
      }
    });
  };

  const applyBackgroundImage = (imageUrl: string | null) => {
    const root = document.documentElement;
    if (imageUrl) {
      root.style.setProperty("--bg-image", `url(${imageUrl})`);
      root.style.setProperty("--bg-overlay", "rgba(0, 0, 0, 0.7)"); // Затемнение для читаемости текста
    } else {
      root.style.setProperty("--bg-image", "none");
      root.style.setProperty("--bg-overlay", "none");
    }
  };

  const loadTheme = () => {
    const saved = localStorage.getItem("customTheme");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        Object.assign(colors, parsed);
      } catch (e) {
        console.error("Failed to load theme", e);
      }
    }

    // Load background image
    const savedBgImage = localStorage.getItem("backgroundImage");
    if (savedBgImage) {
      backgroundImage.value = savedBgImage;
      applyBackgroundImage(savedBgImage);
    }

    applyTheme(colors);
  };

  const saveTheme = (newColors: Partial<ThemeColors>) => {
    Object.assign(colors, newColors);
    localStorage.setItem("customTheme", JSON.stringify(colors));
    applyTheme(colors);
  };

  const resetTheme = () => {
    Object.assign(colors, defaultTheme);
    localStorage.setItem("customTheme", JSON.stringify(colors));
    applyTheme(colors);
    // Reset background image too
    backgroundImage.value = null;
    localStorage.removeItem("backgroundImage");
    applyBackgroundImage(null);
    showColorPicker.value = false;
  };

  const updateColor = (key: keyof ThemeColors, value: string) => {
    saveTheme({ [key]: value });
  };

  const toggleColorPicker = () => {
    console.log(
      "toggleColorPicker called, current state:",
      showColorPicker.value,
    );
    showColorPicker.value = !showColorPicker.value;
    console.log("new state:", showColorPicker.value);
  };

  const setBackgroundImage = (file: File) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        const imageUrl = e.target?.result as string;
        backgroundImage.value = imageUrl;
        localStorage.setItem("backgroundImage", imageUrl);
        applyBackgroundImage(imageUrl);
        resolve(imageUrl);
      };
      reader.onerror = () => {
        reject(new Error("Failed to read file"));
      };
      reader.readAsDataURL(file);
    });
  };

  const removeBackgroundImage = () => {
    backgroundImage.value = null;
    localStorage.removeItem("backgroundImage");
    applyBackgroundImage(null);
  };

  // Load theme immediately
  loadTheme();

  return {
    showColorPicker: showColorPicker,
    colors: readonly(colors),
    backgroundImage: readonly(backgroundImage),
    updateColor,
    resetTheme,
    toggleColorPicker,
    loadTheme,
    setBackgroundImage,
    removeBackgroundImage,
  };
}
