import { mount } from "svelte";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app.css";
import App from "./App.svelte";
import OrchestratorWindow from "./lib/components/OrchestratorWindow.svelte";

const isOverlay = isTauri() && getCurrentWindow().label === "orchestrator";
const Root = isOverlay ? OrchestratorWindow : App;

const app = mount(Root, { target: document.getElementById("app")! });

export default app;
