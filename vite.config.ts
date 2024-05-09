import { defineConfig } from "vite";
import { resolve } from "path";
import { globSync } from "glob";

const base = "/is_cg_gl";

export default defineConfig({
  base,
  root: __dirname,
  plugins: [
    htmlPlugin(),
  ],
  build: {
    outDir: resolve(__dirname, "dist"),
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        ...Object.fromEntries(
          globSync(resolve(__dirname, "*/index.html"))
            .map((file) => {
              const name = file.match(/(\w*)\/index.html/)![1];

              return [name, resolve(__dirname, `${name}/index.html`)];
            })
        ),
      },
    },
  },
  resolve: {
    alias: {
      "wasm": resolve(__dirname, "wasm/pkg"),
    },
  },

});

function htmlPlugin() {
  return {
    name: "html-transform",
    transformIndexHtml(html) {
      return html.replace(/<a href="(.*)">/g, (match, p1) => {
        if (p1.startsWith("/")) {
          return match.replace(p1, `${base}${p1}`);
        }
        return match;
      });
    },
  };
}

