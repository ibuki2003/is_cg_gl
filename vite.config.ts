import { defineConfig } from "vite";
import { resolve } from "path";
import { globSync } from "glob";

export default defineConfig({
  root: __dirname,
  build: {
    outDir: resolve(__dirname, "dist"),
    rollupOptions: {
      input: Object.fromEntries(
        globSync(resolve(__dirname, "*/index.html"))
          .map((file) => {
            const name = file.match(/(.*)\/index.html/)![1];

            return [name, resolve(__dirname, `${name}/index.html`)];
          })
      ),
    },
  },
  resolve: {
    alias: {
      "wasm": resolve(__dirname, "wasm/pkg"),
    },
  },

});

