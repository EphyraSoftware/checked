import { ref } from "vue";
import type { AppClient } from "@holochain/client";

export const holochainClient = ref<AppClient | null>(null);
