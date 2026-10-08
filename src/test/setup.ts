import type { TestingLibraryMatchers } from "@testing-library/jest-dom/matchers";
import * as matchers from "@testing-library/jest-dom/matchers";
import { cleanup } from "@testing-library/react";
import { afterEach, expect } from "vitest";

// The jest-dom Vitest adapter still declares the pre-Vitest 5 Assertion interface.
declare module "vitest" {
  interface Matchers<R, T> extends TestingLibraryMatchers<T, R> {}
}

expect.extend(matchers);

afterEach(() => {
  cleanup();
});
