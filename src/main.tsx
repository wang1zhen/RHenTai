import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./styles/index.css";

async function bootstrap() {
  if (import.meta.env.VITE_MOCK_IPC === "1") {
    const { setupMock } = await import("./dev/mock");
    setupMock();
  }

  const rootElement = document.getElementById("root");
  if (rootElement) {
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );
  }
}

bootstrap();
