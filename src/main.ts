import { mount } from "svelte";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app.css";
import App from "./App.svelte";
import OrchestratorWindow from "./lib/components/OrchestratorWindow.svelte";
import PetHabitat from "./lib/components/PetHabitat.svelte";

const label = isTauri() ? getCurrentWindow().label : "main";
const Root = label === "orchestrator" ? OrchestratorWindow : label === "pets" ? PetHabitat : App;

const app = mount(Root, { target: document.getElementById("app")! });

export default app;
