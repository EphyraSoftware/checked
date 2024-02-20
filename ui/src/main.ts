import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { createPinia } from "pinia";

import { library } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import {
  faClipboard,
  faCheckCircle,
} from "@fortawesome/free-regular-svg-icons";
library.add(faClipboard, faCheckCircle);

const pinia = createPinia();
const app = createApp(App);

app.component("font-awesome-icon", FontAwesomeIcon);
app.use(pinia);
app.mount("#app");
