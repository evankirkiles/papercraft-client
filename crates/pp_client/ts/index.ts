export { PaperClient } from "./client";
export * from "./wasm/client";
import _init from "./wasm/client";

export const init = _init;
