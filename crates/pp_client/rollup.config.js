import typescript from "@rollup/plugin-typescript";
import nodeResolve from "@rollup/plugin-node-resolve";
import copy from "rollup-plugin-copy";

export default {
  input: "ts/index.ts",
  output: {
    dir: "dist",
    format: "esm",
  },
  plugins: [
    typescript({ tsconfig: "tsconfig.json" }),
    nodeResolve(),
    copy({
      targets: [
        { src: "ts/wasm/client.d.ts", dest: "dist/wasm/" },
        { src: "ts/wasm/client_bg.*", dest: "dist/" },
      ],
    }),
  ],
};
