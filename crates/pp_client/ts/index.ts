export * from "./wasm/client";
export { PaperClient } from "./client";
import _init from "./wasm/client";

export const init = _init;
