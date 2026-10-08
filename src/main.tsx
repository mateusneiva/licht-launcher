import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

const root = document.getElementById("root");

if (root === null) {
  throw new Error("The application root element was not found.");
}

createRoot(root).render(<StrictMode>{null}</StrictMode>);
