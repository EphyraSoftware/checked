import { defineStore } from "pinia";
import { computed, ref } from "vue";

export enum Theme {
  Bumblebee = "bumblebee",
  Luxury = "luxury",
}

export const useThemeStore = defineStore("theme", () => {
  const theme = ref<string>(Theme.Bumblebee);

  const stored = localStorage.getItem("theme");
  if (stored) {
    console.log("recover from stored", stored);
    theme.value = stored as Theme;
  }

  const setTheme = (newTheme: Theme) => {
    console.log("do set theme", newTheme);
    theme.value = newTheme;
    localStorage.setItem("theme", newTheme);
  };

  const isDefault = computed(() => {
    return theme.value === Theme.Bumblebee;
  });

  return {
    theme,
    setTheme,
    isDefault,
  };
});
